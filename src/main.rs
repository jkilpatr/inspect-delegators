use cosmos_sdk_proto::cosmos::staking::v1beta1::QueryValidatorsRequest;
use deep_space::Contact;
use std::collections::HashSet;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(10);

async fn get_bugged_delegators() {
    let contact = Contact::new("https://gravitychain.io:9090", TIMEOUT, "gravity").unwrap();
    let mut validators = contact.get_active_validators().await.unwrap();
    let v = contact
        .get_validators_list(QueryValidatorsRequest {
            pagination: None,
            status: "BOND_STATUS_UNBONDING".to_string(),
        })
        .await
        .unwrap();
    validators.extend(v);

    let mut total_delegators = HashSet::new();
    let mut delegators_with_issues = HashSet::new();
    let mut validators_with_issues = HashSet::new();
    let mut total_delegations = 0;
    let mut total_affected_stake: usize = 0;
    for validator in validators {
        let operator_address = validator.operator_address.parse().unwrap();
        let delegations = contact
            .get_validator_delegations(operator_address)
            .await
            .unwrap();
        for d in delegations.iter() {
            total_delegations += 1;
            let delegator_address = d
                .delegation
                .clone()
                .unwrap()
                .delegator_address
                .parse()
                .unwrap();
            total_delegators.insert(delegator_address);
            let res = contact
                .query_delegation_rewards(delegator_address, operator_address)
                .await;
            if res.is_err() {
                delegators_with_issues.insert(delegator_address);
                validators_with_issues.insert(operator_address);
                let stake: usize = d.balance.clone().unwrap().amount.parse().unwrap();
                total_affected_stake += stake;
            }
        }
    }

    println!(
            "Total {} delegations delegators {} bugged delegators {} {:#?} and bugged validators {} {:#?} total affected stake {}",
            total_delegations,
            total_delegators.len(),
            delegators_with_issues.len(),
            delegators_with_issues,
            validators_with_issues.len(),
            validators_with_issues,
            total_affected_stake,
        );
}

#[tokio::main]
async fn main() {
    get_bugged_delegators().await;
}
