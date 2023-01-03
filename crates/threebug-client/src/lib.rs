pub mod client {
    use std::{
        error::Error,
        net::{IpAddr, SocketAddr},
    };

    use bevy_spicy_networking::{NetworkSettings, StandaloneNetworkClient};

    pub struct ThreeBugClient {
        pub client: StandaloneNetworkClient,
        pub ip_address: IpAddr,
        pub port: u16,
        pub socket_address: SocketAddr,
    }

    pub fn default_client() -> Result<ThreeBugClient, Box<dyn Error>> {
        let mut client = StandaloneNetworkClient::new();
        let ip_address = "127.0.0.1".parse().unwrap();
        let port = 9876;

        let socket_address = SocketAddr::new(ip_address, port);

        // info!("Address of the server: {}", socket_address);

        client.connect(
            socket_address,
            NetworkSettings {
                max_packet_length: 10 * 1024 * 1024,
            },
        )?;

        let client = ThreeBugClient {
            client,
            ip_address,
            port,
            socket_address,
        };

        Ok(client)
    }
}

pub fn init() {}

pub mod debug {
    use parry3d::bounding_volume::Aabb;
    use threebug_core::ipc::parry;

    pub fn aabb(aabb: impl Into<Aabb>) {
        let _debug_msg = parry::ParryDebugEntityType::new_aabb_entity(aabb.into());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
