use async_channel::{SendError, Sender};
use futures::future::try_join;
use futures::FutureExt;
use rumqttc::{self, EventLoop, LastWill, MqttOptions, Publish, QoS, Request};
use std::error::Error;
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::task::{self, JoinError, JoinHandle};
use tokio::time::delay_for;

const HOMIE_VERSION: &str = "4.0";
const HOMIE_IMPLEMENTATION: &str = "homie-rs";
const DEFAULT_FIRMWARE_NAME: &str = env!("CARGO_PKG_NAME");
const DEFAULT_FIRMWARE_VERSION: &str = env!("CARGO_PKG_VERSION");
const STATS_INTERVAL: Duration = Duration::from_secs(60);
const REQUESTS_CAP: usize = 10;

/// The data type for a Homie property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Datatype {
    Integer,
    Float,
    Boolean,
    String,
    Enum,
    Color,
}

impl Into<Vec<u8>> for Datatype {
    fn into(self) -> Vec<u8> {
        match self {
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Boolean => "boolean",
            Self::String => "string",
            Self::Enum => "enum",
            Self::Color => "color",
        }
        .into()
    }
}

/// A [property](https://homieiot.github.io/specification/#properties) of a Homie node.
#[derive(Clone, Debug)]
pub struct Property {
    id: String,
    name: String,
    datatype: Datatype,
    unit: Option<String>,
}

impl Property {
    /// Create a new property with the given attributes.
    ///
    /// # Arguments
    /// * `id`: The topic ID for the property. This must be unique per node, and follow the Homie
    ///   [ID format](https://homieiot.github.io/specification/#topic-ids).
    /// * `name`: The human-readable name of the property.
    /// * `datatype`: The data type of the property.
    /// * `unit`: The unit for the property, if any. This may be one of the
    ///   [recommended units](https://homieiot.github.io/specification/#property-attributes), or
    ///   any other custom unit.
    pub fn new(id: &str, name: &str, datatype: Datatype, unit: Option<&str>) -> Property {
        Property {
            id: id.to_owned(),
            name: name.to_owned(),
            datatype: datatype,
            unit: unit.map(|s| s.to_owned()),
        }
    }
}

/// A [node](https://homieiot.github.io/specification/#nodes) of a Homie device.
#[derive(Clone, Debug)]
pub struct Node {
    id: String,
    name: String,
    node_type: String,
    properties: Vec<Property>,
}

impl Node {
    /// Create a new node with the given attributes.
    ///
    /// # Arguments
    /// * `id`: The topic ID for the node. This must be unique per device, and follow the Homie
    ///   [ID format](https://homieiot.github.io/specification/#topic-ids).
    /// * `name`: The human-readable name of the node.
    /// * `type`: The type of the node. This is an arbitrary string.
    /// * `property`: The properties of the node. There should be at least one.
    pub fn new(id: String, name: String, node_type: String, properties: Vec<Property>) -> Node {
        Node {
            id,
            name,
            node_type,
            properties,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    NotStarted,
    Init,
    Ready,
}

/// A Homie [device](https://homieiot.github.io/specification/#devices). This corresponds to a
/// single MQTT connection.
pub struct HomieDevice {
    requests_tx: Sender<Request>,
    device_base: String,
    device_name: String,
    firmware_name: String,
    firmware_version: String,
    nodes: Vec<Node>,
    state: State,
}

impl HomieDevice {
    /// Create a new Homie device, connect to the MQTT server, and start a task to handle the MQTT
    /// connection.
    ///
    /// # Arguments
    /// * `device_base`: The base topic ID for the device, including the Homie base topic. This
    ///   might be something like "homie/my-device-id" if you are using the default Homie
    ///   [base topic](https://homieiot.github.io/specification/#base-topic). This must be
    ///   unique per MQTT server.
    /// * `device_name`: The human-readable name of the device.
    /// * `mqtt_options`: Options for the MQTT connection, including which server to connect to.
    ///
    /// # Return value
    /// A pair of the `HomieDevice` itself, and a `Future` for the tasks which handle the MQTT
    /// connection. You should join on this future to handle any errors it returns.
    pub async fn spawn(
        device_base: &str,
        device_name: &str,
        mut mqtt_options: MqttOptions,
    ) -> Result<
        (
            HomieDevice,
            impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>>,
        ),
        SendError<Request>,
    > {
        mqtt_options.set_last_will(LastWill {
            topic: format!("{}/$state", device_base),
            message: "lost".to_string(),
            qos: QoS::AtLeastOnce,
            retain: true,
        });
        let mut event_loop = EventLoop::new(mqtt_options, REQUESTS_CAP).await;

        let mut homie = HomieDevice::new(
            event_loop.handle(),
            device_base.to_string(),
            device_name.to_string(),
        );

        let stats = HomieStats::new(event_loop.handle(), device_base.to_string());

        // This needs to be spawned before we wait for anything to be sent, as the start() calls below do.
        let event_task: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> =
            task::spawn(async move {
                loop {
                    let (incoming, outgoing) = event_loop.poll().await?;
                    log::trace!("Incoming = {:?}, Outgoing = {:?}", incoming, outgoing);
                }
            });

        stats.start().await?;
        homie.start().await?;

        let stats_task: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> =
            task::spawn(stats.run());
        let join_handle = try_join_handles(event_task, stats_task).map(|r| r.map(|((), ())| ()));

        Ok((homie, join_handle))
    }

    fn new(requests_tx: Sender<Request>, device_base: String, device_name: String) -> HomieDevice {
        HomieDevice {
            requests_tx,
            device_base,
            device_name,
            firmware_name: DEFAULT_FIRMWARE_NAME.to_string(),
            firmware_version: DEFAULT_FIRMWARE_VERSION.to_string(),
            nodes: vec![],
            state: State::NotStarted,
        }
    }

    async fn start(&mut self) -> Result<(), SendError<Request>> {
        assert_eq!(self.state, State::NotStarted);
        publish_retained(
            &self.requests_tx,
            format!("{}/$homie", self.device_base),
            HOMIE_VERSION,
        )
        .await?;
        // TODO: Send $localip and $mac too.
        publish_retained(
            &self.requests_tx,
            format!("{}/$fw/name", self.device_base),
            self.firmware_name.as_str(),
        )
        .await?;
        publish_retained(
            &self.requests_tx,
            format!("{}/$fw/version", self.device_base),
            self.firmware_version.as_str(),
        )
        .await?;
        publish_retained(
            &self.requests_tx,
            format!("{}/$extensions", self.device_base),
            "org.homie.legacy-firmware:0.1.1:[4.x],org.homie.legacy-stats:0.1.1:[4.x]",
        )
        .await?;
        publish_retained(
            &self.requests_tx,
            format!("{}/$implementation", self.device_base),
            HOMIE_IMPLEMENTATION,
        )
        .await?;
        publish_retained(
            &self.requests_tx,
            format!("{}/$name", self.device_base),
            self.device_name.as_str(),
        )
        .await?;
        publish_retained(
            &self.requests_tx,
            format!("{}/$state", self.device_base),
            "init",
        )
        .await?;
        self.state = State::Init;
        Ok(())
    }

    /// Set the firmware name and version to be advertised for the Homie device.
    ///
    /// If this is not set, it will default to the cargo package name and version.
    #[allow(dead_code)]
    pub fn set_firmware(&mut self, firmware_name: &str, firmware_version: &str) {
        assert_eq!(self.state, State::NotStarted);
        self.firmware_name = firmware_name.to_string();
        self.firmware_version = firmware_version.to_string();
    }

    pub async fn add_node(&mut self, node: Node) -> Result<(), SendError<Request>> {
        // First check that there isn't already a node with the same ID.
        if self.nodes.iter().any(|n| n.id == node.id) {
            panic!("Tried to add node with duplicate ID: {:?}", node);
        }
        self.nodes.push(node);
        // `node` was moved into the `nodes` vector, but we can safely get a reference to it because
        // nothing else can modify `nodes` in the meantime.
        let node = &self.nodes[self.nodes.len() - 1];

        self.publish_node(&node).await?;
        self.publish_nodes().await
    }

    pub async fn remove_node(&mut self, node_id: &str) -> Result<(), SendError<Request>> {
        self.nodes.retain(|n| n.id != node_id);
        self.publish_nodes().await
    }

    async fn publish_node(&self, node: &Node) -> Result<(), SendError<Request>> {
        let node_base = format!("{}/{}", self.device_base, node.id);
        publish_retained(
            &self.requests_tx,
            format!("{}/$name", node_base),
            node.name.as_str(),
        )
        .await?;
        publish_retained(
            &self.requests_tx,
            format!("{}/$type", node_base),
            node.node_type.as_str(),
        )
        .await?;
        let mut property_ids: Vec<&str> = vec![];
        for property in &node.properties {
            property_ids.push(&property.id);
            publish_retained(
                &self.requests_tx,
                format!("{}/{}/$name", node_base, property.id),
                property.name.as_str(),
            )
            .await?;
            publish_retained(
                &self.requests_tx,
                format!("{}/{}/$datatype", node_base, property.id),
                property.datatype,
            )
            .await?;
            if let Some(unit) = &property.unit {
                publish_retained(
                    &self.requests_tx,
                    format!("{}/{}/$unit", node_base, property.id),
                    unit.as_str(),
                )
                .await?;
            }
        }
        publish_retained(
            &self.requests_tx,
            format!("{}/$properties", node_base),
            property_ids.join(","),
        )
        .await?;
        Ok(())
    }

    async fn publish_nodes(&mut self) -> Result<(), SendError<Request>> {
        let node_ids = self
            .nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<&str>>()
            .join(",");
        publish_retained(
            &self.requests_tx,
            format!("{}/$nodes", self.device_base),
            node_ids,
        )
        .await
    }

    pub async fn ready(&mut self) -> Result<(), SendError<Request>> {
        assert_eq!(self.state, State::Init);
        self.state = State::Ready;

        publish_retained(
            &self.requests_tx,
            format!("{}/$state", self.device_base),
            "ready",
        )
        .await?;

        Ok(())
    }

    pub async fn publish_value(
        &self,
        node_id: &str,
        property_id: &str,
        value: impl ToString,
    ) -> Result<(), SendError<Request>> {
        publish_retained(
            &self.requests_tx,
            format!("{}/{}/{}", self.device_base, node_id, property_id),
            value.to_string(),
        )
        .await
    }
}

/// Legacy stats extension.
struct HomieStats {
    requests_tx: Sender<Request>,
    device_base: String,
    start_time: Instant,
}

impl HomieStats {
    fn new(requests_tx: Sender<Request>, device_base: String) -> Self {
        let now = Instant::now();
        Self {
            requests_tx,
            device_base,
            start_time: now,
        }
    }

    /// Send initial topics.
    async fn start(&self) -> Result<(), SendError<Request>> {
        publish_retained(
            &self.requests_tx,
            format!("{}/$stats/interval", self.device_base),
            STATS_INTERVAL.as_secs().to_string(),
        )
        .await
    }

    /// Periodically send stats.
    async fn run(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            let uptime = Instant::now() - self.start_time;
            publish_retained(
                &self.requests_tx,
                format!("{}/$stats/uptime", self.device_base),
                uptime.as_secs().to_string(),
            )
            .await?;
            delay_for(STATS_INTERVAL).await;
        }
    }
}

async fn publish_retained(
    requests_tx: &Sender<Request>,
    name: String,
    value: impl Into<Vec<u8>>,
) -> Result<(), SendError<Request>> {
    let mut publish = Publish::new(name, QoS::AtLeastOnce, value);
    publish.set_retain(true);
    requests_tx.send(publish.into()).await
}

fn try_join_handles<A, B, E>(
    a: JoinHandle<Result<A, E>>,
    b: JoinHandle<Result<B, E>>,
) -> impl Future<Output = Result<(A, B), E>>
where
    E: From<JoinError>,
{
    try_join(a.map(|res| Ok(res??)), b.map(|res| Ok(res??)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_channel::Receiver;

    fn make_test_device() -> (HomieDevice, Receiver<Request>) {
        let (requests_tx, requests_rx) = async_channel::unbounded();
        let device = HomieDevice::new(
            requests_tx,
            "homie/test-device".to_string(),
            "Test device".to_string(),
        );
        (device, requests_rx)
    }

    #[tokio::test]
    #[should_panic(expected = "Tried to add node with duplicate ID")]
    async fn add_node_fails_given_duplicate_id() {
        let (mut device, rx) = make_test_device();

        device
            .add_node(Node::new(
                "id".to_string(),
                "Name".to_string(),
                "type".to_string(),
                vec![],
            ))
            .await
            .unwrap();
        device
            .add_node(Node::new(
                "id".to_string(),
                "Name 2".to_string(),
                "type2".to_string(),
                vec![],
            ))
            .await
            .unwrap();
        // Need to keep rx alive until here so that the channel isn't closed.
        drop(rx);
    }

    #[tokio::test]
    #[should_panic(expected = "NotStarted")]
    async fn ready_fails_if_called_before_start() {
        let (mut device, rx) = make_test_device();

        device.ready().await.unwrap();

        // Need to keep rx alive until here so that the channel isn't closed.
        drop(rx);
    }

    #[tokio::test]
    async fn start_succeeds_with_no_nodes() {
        let (mut device, rx) = make_test_device();

        device.start().await.unwrap();
        device.ready().await.unwrap();

        // Need to keep rx alive until here so that the channel isn't closed.
        drop(rx);
    }

    #[tokio::test]
    async fn set_firmware_succeeds_before_start() {
        let (mut device, rx) = make_test_device();

        device.set_firmware("firmware_name", "firmware_version");

        device.start().await.unwrap();
        device.ready().await.unwrap();

        // Need to keep rx alive until here so that the channel isn't closed.
        drop(rx);
    }

    #[tokio::test]
    #[should_panic(expected = "NotStarted")]
    async fn set_firmware_fails_after_start() {
        let (mut device, rx) = make_test_device();

        device.start().await.unwrap();

        device.set_firmware("firmware_name", "firmware_version");

        // Need to keep rx alive until here so that the channel isn't closed.
        drop(rx);
    }

    #[tokio::test]
    async fn add_node_succeeds_before_and_after_start() {
        let (mut device, rx) = make_test_device();

        device
            .add_node(Node::new(
                "id".to_string(),
                "Name".to_string(),
                "type".to_string(),
                vec![],
            ))
            .await
            .unwrap();

        device.start().await.unwrap();
        device.ready().await.unwrap();

        // Add another node after starting.
        device
            .add_node(Node::new(
                "id2".to_string(),
                "Name 2".to_string(),
                "type2".to_string(),
                vec![],
            ))
            .await
            .unwrap();

        // Need to keep rx alive until here so that the channel isn't closed.
        drop(rx);
    }

    /// Add a node, remove it, and add it back again.
    #[tokio::test]
    async fn add_node_succeeds_after_remove() {
        let (mut device, rx) = make_test_device();

        device
            .add_node(Node::new(
                "id".to_string(),
                "Name".to_string(),
                "type".to_string(),
                vec![],
            ))
            .await
            .unwrap();

        device.remove_node("id").await.unwrap();

        // Adding it back shouldn't give an error.
        device
            .add_node(Node::new(
                "id".to_string(),
                "Name".to_string(),
                "type".to_string(),
                vec![],
            ))
            .await
            .unwrap();

        // Need to keep rx alive until here so that the channel isn't closed.
        drop(rx);
    }
}
