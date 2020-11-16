import React from "react"
import {ListGroup} from 'react-bootstrap';
import NavBar from "../components/navbar"
import Config from "../config"
import auth from "../js/auth"
import { navigate } from "gatsby"


export default function home() {

    // If no active user go to login screen
    auth.validateSession().then((results)=>{
        const [isValidSession, username] = results; 
        if (!isValidSession) {
            if (username) {
                console.log(`!!!!!!! Non Active User: ${username}`);
            } else {
                console.log(`No User Found`);
            }

            navigate('/signon', {replace: true});
            return;
        }
        console.log(`Active User: ${username}`)
    });
    
    
    const addNewOrder = ()=>{
        console.log("Add new order");
    };
    
    return (
        <div>
            <NavBar/>
            <div>You have sold</div>
            <div>Summary X</div>
            <div>Summary Y</div>
            <div>Summary Z</div>
            <div>Summary R</div>
            
            <button type="button"
                    className="btn btn-outline-light add-order-btn"
                    onClick={addNewOrder}>
                +
            </button>
        </div>
    );
}
