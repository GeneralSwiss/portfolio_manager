use crate::domain::Leg;
use crate::{Error, Greeks, Position};
use ibapi::contracts::SecurityType;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

impl TryFrom<ibapi::accounts::Position> for Position {
    type Error = Error;
    fn try_from(value: ibapi::accounts::Position) -> Result<Self, Self::Error> {
        let _average_cost = value.average_cost;
        let quantity = value.position;
        let underlying_symbol = value.contract.symbol.clone();
        let security_type = value.contract.security_type;
        let expiration_date = value.contract.last_trade_date_or_contract_month;
        let strike = value.contract.strike;
        let _right = value.contract.right;
        let contract_symbol = value.contract.symbol.clone();
        let _combo_legs_description = value.contract.combo_legs_description;
        let legs = value.contract.combo_legs;

        let _legs: Vec<Leg> = legs
            .iter()
            .map(|_their_leg| match security_type {
                SecurityType::Stock => Ok(Leg::stock(contract_symbol.clone(), quantity)),
                SecurityType::Option => Ok(Leg::option(
                    contract_symbol.clone(),
                    underlying_symbol.clone(),
                    quantity,
                    Decimal::from_f64(strike)
                        .ok_or(Error::MalformedDecimalError(strike.to_string()))?,
                    expiration_date.parse()?,
                    Greeks::default(),
                )),
                _ => Err(Error::UnknownLegType(security_type.to_string())),
            })
            .collect::<Result<Vec<_>, _>>()?;
        todo!()
    }
}
