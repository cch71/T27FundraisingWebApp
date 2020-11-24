import React from "react"
import {orderDb} from "../js/ordersdb"
import currency from "currency.js"

export default class OrderItem extends React.Component {
    // constructor(props) {
    //     super(props);
    // }


    render() {
        const deliveryDate = this.props.deliveryDate;
        const label = ("donation" === deliveryDate)? 'Donation' : `Order for ${deliveryDate}`;

        const currentOrder = orderDb.getCurrentOrder();
        const foundOrder = currentOrder.deliverables.get(deliveryDate);

        const newTag = `new-${deliveryDate}`;
        const foundTag = `found-${deliveryDate}`
        
        const USD = (value) => currency(value, { symbol: "$", precision: 2 });

        const onDeleteOrder = (event)=>{
            const btn = event.currentTarget;
            
            console.log(`Deleting Order for ${btn.dataset.deliverydate}`);

            currentOrder.deliverables.delete(btn.dataset.deliverydate);
            document.getElementById(newTag).style.display = "block";
            document.getElementById(foundTag).style.display = "none";

            if (this.props.onDelete) {
                this.props.onDelete();
            }
        }


        
        const newOrderStyle = (undefined===foundOrder)?  {display: 'block'}:{display: 'none'};
        const foundOrderStyle = (undefined!==foundOrder)? {display: 'block'}:{display: 'none'};
        let orderTotalStr = '';
        if (undefined !== foundOrder) {
            orderTotalStr = `${label} Cost: ${USD(foundOrder.totalDue).format()} `;
        }
        
        return(
            <div>
                <div id={newTag} style={newOrderStyle} >
                    <button className="btn btn-primary" type="button" onClick={this.props.onClick}
                            data-deliverydate={deliveryDate} >
                        {label}
                    </button>
                </div>
                <div id={foundTag} style={foundOrderStyle}>
                    {orderTotalStr}
                    <button className="btn btn-outline-danger mx-1 float-right"
                            data-deliverydate={deliveryDate} onClick={onDeleteOrder}>X</button>
                    <button className="btn btn-outline-info float-right"
                            data-deliverydate={deliveryDate} onClick={this.props.onClick}>I</button>
                </div>
            </div>
        );
    }
}
