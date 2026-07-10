use data_model::{
    get_active_order_state,
    gql_utils::{GraphQlReq, gql_escape, make_gql_request},
};
use tracing::{error, info};

fn gen_submit_active_order_req_str() -> Result<String, Box<dyn std::error::Error>> {
    let order_state = get_active_order_state().ok_or("No active order to submit")?;
    if !order_state.is_dirty {
        info!("Order doesn't need updating so not submitting");
        return Ok("".to_string());
    }

    let order = &order_state.order;

    let mut query = String::with_capacity(1024 * 32);
    query.push_str("mutation {\n");
    if order_state.is_new_order {
        query.push_str("\t createMulchOrder(order: {\n");
    } else {
        query.push_str("\t updateMulchOrder(order: {\n");
    }

    query.push_str(&format!(
        "\t\t orderId: \"{}\"\n",
        gql_escape(order.order_id.trim())
    ));
    query.push_str(&format!(
        "\t\t ownerId: \"{}\"\n",
        gql_escape(order.order_owner_id.trim())
    ));

    if let Some(value) = order.comments.as_ref() {
        query.push_str(&format!("\t\t comments: \"{}\"\n", gql_escape(value.trim())));
    }

    if let Some(value) = order.special_instructions.as_ref() {
        query.push_str(&format!(
            "\t\t specialInstructions: \"{}\"\n",
            gql_escape(value.trim())
        ));
    }

    if let Some(value) = order.is_verified.as_ref() {
        query.push_str(&format!("\t\t isVerified: {value}\n"));
    }

    if let Some(value) = order.amount_total_collected.as_ref() {
        query.push_str(&format!(
            "\t\t amountTotalCollected: \"{}\"\n",
            value.trim()
        ));
    } else {
        if !order.will_collect_money_later.unwrap_or(false) {
            error!("Total collected is zero. will collect later should be true");
        }
        query.push_str("\t\t willCollectMoneyLater: true\n");
    }

    if let Some(value) = order.amount_from_donations.as_ref() {
        query.push_str(&format!("\t\t amountFromDonations: \"{}\"\n", value.trim()));
    }

    if let Some(value) = order.amount_from_purchases.as_ref() {
        query.push_str(&format!("\t\t amountFromPurchases: \"{}\"\n", value.trim()));

        let mut purchases = Vec::new();
        for (product_id, info) in order.purchases.as_ref().unwrap() {
            let mut purchase_str = String::new();
            purchase_str.push_str("\t\t\t {\n");
            purchase_str.push_str(&format!("\t\t\t\t productId: \"{}\"\n", product_id.trim()));
            purchase_str.push_str(&format!("\t\t\t\t numSold: {}\n", info.num_sold));
            purchase_str.push_str(&format!(
                "\t\t\t\t amountCharged: \"{}\"\n",
                info.amount_charged.trim()
            ));
            purchase_str.push_str("\t\t\t }\n");
            purchases.push(purchase_str);
        }

        query.push_str("\t\t purchases: [\n");
        query.push_str(&purchases.join(","));
        query.push_str("\t\t ]\n");
    }

    if let Some(value) = order.amount_cash_collected.as_ref() {
        query.push_str(&format!(
            "\t\t amountFromCashCollected: \"{}\"\n",
            value.trim()
        ));
    }

    if let Some(value) = order.amount_checks_collected.as_ref() {
        query.push_str(&format!(
            "\t\t amountFromChecksCollected: \"{}\"\n",
            value.trim()
        ));
        query.push_str(&format!(
            "\t\t checkNumbers: \"{}\"\n",
            gql_escape(order.check_numbers.as_ref().unwrap().trim())
        ));
    }

    query.push_str(&format!("\t\t deliveryId: {}\n", order.delivery_id));

    query.push_str("\t\t customer: {\n");
    query.push_str(&format!(
        "\t\t\t name: \"{}\"\n",
        gql_escape(order.customer.name.trim())
    ));
    query.push_str(&format!(
        "\t\t\t addr1: \"{}\"\n",
        gql_escape(order.customer.addr1.trim())
    ));
    if let Some(value) = order.customer.addr2.as_ref() {
        query.push_str(&format!("\t\t\t addr2: \"{}\"\n", gql_escape(value.trim())));
    }
    if let Some(value) = order.customer.city.as_ref() {
        query.push_str(&format!("\t\t\t city: \"{}\"\n", gql_escape(value.trim())));
    }
    if let Some(value) = order.customer.zipcode.as_ref() {
        query.push_str(&format!("\t\t\t zipcode: {value}\n"));
    }
    query.push_str(&format!(
        "\t\t\t phone: \"{}\"\n",
        gql_escape(order.customer.phone.trim())
    ));
    if let Some(value) = order.customer.email.as_ref() {
        query.push_str(&format!("\t\t email: \"{}\"\n", gql_escape(value.trim())));
    }
    query.push_str(&format!(
        "\t\t\t neighborhood: \"{}\"\n",
        gql_escape(
            order
                .customer
                .neighborhood
                .as_ref()
                .unwrap_or(&"".to_string())
                .trim()
        )
    ));
    query.push_str("\t\t }\n");

    query.push_str("\t})\n");
    query.push('}');
    Ok(query)
}

pub async fn submit_active_order() -> Result<(), Box<dyn std::error::Error>> {
    let query = gen_submit_active_order_req_str()?;

    if query.is_empty() {
        // If a query wasn't generated, then we don't need to submit it
        return Ok(());
    }

    info!("Submitting Request:\n{}", &query);

    let req = GraphQlReq::new(query);
    make_gql_request::<serde_json::Value>(&req)
        .await
        .map(|_| ())
}
