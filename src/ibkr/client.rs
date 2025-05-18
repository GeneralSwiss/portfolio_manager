use crate::Result;
use crate::ibkr::IbClientConfig;
use ibapi::Client;

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
