use crate::Result;
use crate::ibkr::IbClientConfig;
use ibapi::Client;
use ibapi::accounts::PositionUpdate;

#[derive(Debug)]
pub struct IbkrClient {
    config: IbClientConfig,
    core: Client,
}

impl TryFrom<IbClientConfig> for IbkrClient {
    type Error = crate::Error;
    fn try_from(config: IbClientConfig) -> Result<Self> {
        Ok(Self {
            core: Client::connect(&config.get_address(), config.client_id)?,
            config,
        })
    }
}

impl IbkrClient {
    pub fn get_positions(&self) -> Result<()> {
        let subscription = self.core.positions().expect("Positions to be returned");
        for position_response in subscription.iter() {
            match position_response {
                PositionUpdate::Position(position) => {
                    let _position = dbg!(position);
                }
                PositionUpdate::PositionEnd => {
                    println!("PositionEnd");
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ibkr_client() {
        let config = IbClientConfig::default();
        let client = IbkrClient::try_from(config).unwrap();
        client.get_positions().unwrap();
    }
}
