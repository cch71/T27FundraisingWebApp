import React from "react"
import {orderDb} from "../js/ordersdb"
import currency from "currency.js"

export default class OrderItem extends React.Component {
    // constructor(props) {
    //     super(props);
    // }


    render() {
        const deliveryId = this.props.deliveryId;
        const deliveryLabel = this.props.deliveryLabel;
        const label = ("donation" === deliveryId)? 'Donation' : `Order for ${deliveryLabel}`;

        const currentOrder = orderDb.getActiveOrder();
        const foundOrder = currentOrder.orderByDelivery.get(deliveryId);

        const newTag = `new-${deliveryId}`;
        const foundTag = `found-${deliveryId}`

        //console.log(`Handling Order Item: ${deliveryId}:${JSON.stringify(foundOrder)}`);
        
        const USD = (value) => currency(value, { symbol: "$", precision: 2 });

        const onDeleteOrder = (event)=>{
            const btn = event.currentTarget;
            
            console.log(`Deleting Order for ${btn.dataset.deliverylabel}`);

            currentOrder.orderByDelivery.delete(btn.dataset.deliveryid);
            document.getElementById(newTag).style.display = "block";
            document.getElementById(foundTag).style.display = "none";

            if (this.props.onDelete) {
                this.props.onDelete(event);
            }
        }


        
        const newOrderStyle = (undefined===foundOrder)?  {display: 'block'}:{display: 'none'};
        const foundOrderStyle = (undefined!==foundOrder)? {display: 'block'}:{display: 'none'};
        let orderTotalStr = '';
        if (undefined !== foundOrder) {
            orderTotalStr = `${label} Cost: ${USD(foundOrder.amountDue).format()} `;
        }
        
        return(
            <div>
                <div id={newTag} style={newOrderStyle} >
                    <button className="btn btn-primary" type="button" onClick={this.props.onClick}
                            data-deliverylabel={deliveryLabel}  data-deliveryid={deliveryId}>
                        {label}
                    </button>
                </div>
                <div id={foundTag} style={foundOrderStyle}>
                    {orderTotalStr}
                    <button className="btn btn-outline-danger mx-1 float-right order-edt-btn"
                            data-deliverylabel={deliveryLabel} data-deliveryid={deliveryId}
                            onClick={onDeleteOrder}>
                        <span>&#10005;</span>
                    </button>
                    <button className="btn btn-outline-info float-right order-edt-btn"
                            data-deliverylabel={deliveryLabel} data-deliveryid={deliveryId}
                            onClick={this.props.onClick}>
                        <span>&#9999;</span>
                    </button>
                </div>
            </div>
        );
    }
}
