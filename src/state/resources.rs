use serde::{Deserialize, Serialize};

use crate::data::GameData;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceInventory {
    pub stacks: Vec<ResourceStack>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceStack {
    pub resource_id: String,
    pub amount: i32,
}

impl ResourceInventory {
    pub fn from_data(data: &GameData) -> Self {
        Self {
            stacks: data
                .resources
                .iter()
                .map(|resource| ResourceStack {
                    resource_id: resource.id.clone(),
                    amount: resource.starting_amount,
                })
                .collect(),
        }
    }

    pub fn amount(&self, resource_id: &str) -> i32 {
        self.stacks
            .iter()
            .find(|stack| stack.resource_id == resource_id)
            .map(|stack| stack.amount)
            .unwrap_or(0)
    }

    pub fn add(&mut self, resource_id: &str, amount: i32) {
        if let Some(stack) = self
            .stacks
            .iter_mut()
            .find(|stack| stack.resource_id == resource_id)
        {
            stack.amount += amount;
            return;
        }

        self.stacks.push(ResourceStack {
            resource_id: resource_id.to_owned(),
            amount,
        });
    }

    pub fn spend(&mut self, costs: &[(String, i32)]) -> Result<(), Vec<(String, i32)>> {
        let missing = self.missing_costs(costs);
        if !missing.is_empty() {
            return Err(missing);
        }

        for (resource_id, amount) in costs {
            if let Some(stack) = self
                .stacks
                .iter_mut()
                .find(|stack| stack.resource_id == *resource_id)
            {
                stack.amount -= amount;
            }
        }
        Ok(())
    }

    pub fn missing_costs(&self, costs: &[(String, i32)]) -> Vec<(String, i32)> {
        costs
            .iter()
            .filter_map(|(resource_id, amount)| {
                let current = self.amount(resource_id);
                if current < *amount {
                    Some((resource_id.clone(), amount - current))
                } else {
                    None
                }
            })
            .collect()
    }
}
