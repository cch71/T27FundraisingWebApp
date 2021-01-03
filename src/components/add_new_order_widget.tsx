import React from 'react'
import { navigate } from "gatsby";
import {orderDb} from "../js/ordersdb";
import bootstrapIconSprite from "bootstrap-icons/bootstrap-icons.svg";

const addOrderImg = bootstrapIconSprite + "#plus-square-fill"

const AddNewOrderWidget = () => {
    const addNewOrder = ()=>{
        console.log("Add new order");
        orderDb.newActiveOrder();
        navigate('/order_step_1/');
    };

    // float-end me-3 my-1
    return (
        <div className="add-order-widget">
            <label>Add New Order</label>
            <button type="button"
                    className="btn btn-outline-primary add-order-btn"
                    onClick={addNewOrder}>
                <svg className="bi" fill="currentColor">
                    <use xlinkHref={addOrderImg}/>
                </svg>
            </button>
        </div>
    );
}

export default AddNewOrderWidget;
