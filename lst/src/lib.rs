use async_graphql::{scalar, Request, Response};
use fungible::FungibleTokenAbi;
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{AccountOwner, Amount, ApplicationId, ChainId, ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

pub struct LstAbi;

impl ContractAbi for LstAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for LstAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Parameters {
    pub protocol_lst: ApplicationId<FungibleTokenAbi>,
}
impl Parameters {
    pub fn get_protocol_lst(&self) -> ApplicationId<FungibleTokenAbi> {
        self.protocol_lst
    }
}

scalar!(Parameters);

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    NewLst {
        token_id: ApplicationId,
    },
    StakeNative {
        user: AccountOwner,
        amount: Amount,
        lst_type_out: ApplicationId,
    },
    StakeLst {
        user: AccountOwner,
        amount: Amount,
        lst_type_in: ApplicationId,
    },

    Unstake {
        owner: AccountOwner,
        amount: Amount,
    },
    Swap {
        user: AccountOwner,
        amount_in: Amount,
        lst_type_in: ApplicationId,
        lst_type_out: ApplicationId,
    },
    Test,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    StakeLocalAccount {
        owner: AccountOwner,
        amount: Amount,
    },
    StakeNative {
        user: AccountOwner,
        amount: Amount,
        lst_type_out: ApplicationId,
        user_chain_id: ChainId,
    },
    StakeLst {
        user: AccountOwner,
        amount_in: Amount,
        user_chain_id: ChainId,
    },
    Swap {
        user: AccountOwner,
        amount_in: Amount,
        user_chain_id: ChainId,
        lst_type_in: ApplicationId,
        lst_type_out: ApplicationId,
    },
}
