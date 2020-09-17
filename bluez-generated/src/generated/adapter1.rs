// This code was autogenerated with `dbus-codegen-rust --file=specs/org.bluez.Adapter1.xml --interfaces=org.bluez.Adapter1 --client=nonblock --methodtype=none`, see https://github.com/diwic/dbus-rs
use dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus::nonblock;

pub trait OrgBluezAdapter1 {
    fn start_discovery(&self) -> nonblock::MethodReply<()>;
    fn set_discovery_filter(
        &self,
        properties: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
    ) -> nonblock::MethodReply<()>;
    fn stop_discovery(&self) -> nonblock::MethodReply<()>;
    fn remove_device(&self, device: dbus::Path) -> nonblock::MethodReply<()>;
    fn get_discovery_filters(&self) -> nonblock::MethodReply<Vec<String>>;
    fn address(&self) -> nonblock::MethodReply<String>;
    fn address_type(&self) -> nonblock::MethodReply<String>;
    fn name(&self) -> nonblock::MethodReply<String>;
    fn alias(&self) -> nonblock::MethodReply<String>;
    fn set_alias(&self, value: String) -> nonblock::MethodReply<()>;
    fn class(&self) -> nonblock::MethodReply<u32>;
    fn powered(&self) -> nonblock::MethodReply<bool>;
    fn set_powered(&self, value: bool) -> nonblock::MethodReply<()>;
    fn discoverable(&self) -> nonblock::MethodReply<bool>;
    fn set_discoverable(&self, value: bool) -> nonblock::MethodReply<()>;
    fn discoverable_timeout(&self) -> nonblock::MethodReply<u32>;
    fn set_discoverable_timeout(&self, value: u32) -> nonblock::MethodReply<()>;
    fn pairable(&self) -> nonblock::MethodReply<bool>;
    fn set_pairable(&self, value: bool) -> nonblock::MethodReply<()>;
    fn pairable_timeout(&self) -> nonblock::MethodReply<u32>;
    fn set_pairable_timeout(&self, value: u32) -> nonblock::MethodReply<()>;
    fn discovering(&self) -> nonblock::MethodReply<bool>;
    fn uuids(&self) -> nonblock::MethodReply<Vec<String>>;
    fn modalias(&self) -> nonblock::MethodReply<String>;
}

impl<'a, T: nonblock::NonblockReply, C: ::std::ops::Deref<Target = T>> OrgBluezAdapter1
    for nonblock::Proxy<'a, C>
{
    fn start_discovery(&self) -> nonblock::MethodReply<()> {
        self.method_call("org.bluez.Adapter1", "StartDiscovery", ())
    }

    fn set_discovery_filter(
        &self,
        properties: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
    ) -> nonblock::MethodReply<()> {
        self.method_call("org.bluez.Adapter1", "SetDiscoveryFilter", (properties,))
    }

    fn stop_discovery(&self) -> nonblock::MethodReply<()> {
        self.method_call("org.bluez.Adapter1", "StopDiscovery", ())
    }

    fn remove_device(&self, device: dbus::Path) -> nonblock::MethodReply<()> {
        self.method_call("org.bluez.Adapter1", "RemoveDevice", (device,))
    }

    fn get_discovery_filters(&self) -> nonblock::MethodReply<Vec<String>> {
        self.method_call("org.bluez.Adapter1", "GetDiscoveryFilters", ())
            .and_then(|r: (Vec<String>,)| Ok(r.0))
    }

    fn address(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "Address",
        )
    }

    fn address_type(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "AddressType",
        )
    }

    fn name(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "Name",
        )
    }

    fn alias(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "Alias",
        )
    }

    fn class(&self) -> nonblock::MethodReply<u32> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "Class",
        )
    }

    fn powered(&self) -> nonblock::MethodReply<bool> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "Powered",
        )
    }

    fn discoverable(&self) -> nonblock::MethodReply<bool> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "Discoverable",
        )
    }

    fn discoverable_timeout(&self) -> nonblock::MethodReply<u32> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "DiscoverableTimeout",
        )
    }

    fn pairable(&self) -> nonblock::MethodReply<bool> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "Pairable",
        )
    }

    fn pairable_timeout(&self) -> nonblock::MethodReply<u32> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "PairableTimeout",
        )
    }

    fn discovering(&self) -> nonblock::MethodReply<bool> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "Discovering",
        )
    }

    fn uuids(&self) -> nonblock::MethodReply<Vec<String>> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "UUIDs",
        )
    }

    fn modalias(&self) -> nonblock::MethodReply<String> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.bluez.Adapter1",
            "Modalias",
        )
    }

    fn set_alias(&self, value: String) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            &self,
            "org.bluez.Adapter1",
            "Alias",
            value,
        )
    }

    fn set_powered(&self, value: bool) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            &self,
            "org.bluez.Adapter1",
            "Powered",
            value,
        )
    }

    fn set_discoverable(&self, value: bool) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            &self,
            "org.bluez.Adapter1",
            "Discoverable",
            value,
        )
    }

    fn set_discoverable_timeout(&self, value: u32) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            &self,
            "org.bluez.Adapter1",
            "DiscoverableTimeout",
            value,
        )
    }

    fn set_pairable(&self, value: bool) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            &self,
            "org.bluez.Adapter1",
            "Pairable",
            value,
        )
    }

    fn set_pairable_timeout(&self, value: u32) -> nonblock::MethodReply<()> {
        <Self as nonblock::stdintf::org_freedesktop_dbus::Properties>::set(
            &self,
            "org.bluez.Adapter1",
            "PairableTimeout",
            value,
        )
    }
}
