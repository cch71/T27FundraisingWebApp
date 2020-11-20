import React from "react"
import {Button} from "react-bootstrap"
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
                    <Button variant="primary" type="button" onClick={this.props.onClick}
                            data-deliverydate={deliveryDate} >
                        {label}
                    </Button>
                </div>
                <div id={foundTag} style={foundOrderStyle}>
                    {orderTotalStr}
                    <Button variant="outline-danger" className="mx-1 float-right"
                            data-deliverydate={deliveryDate} onClick={onDeleteOrder}>X</Button>
                    <Button variant="outline-info" className="float-right"
                            data-deliverydate={deliveryDate} onClick={this.props.onClick}>I</Button>
                </div>
            </div>
        );
    }
}
