import React, { useState, useEffect } from 'react'
import { Link } from 'gatsby'
import auth from "../js/auth"
import t27patch from "../../static/t27patch.jpg"
import {orderDb} from "../js/ordersdb"

const NavBar = () => {
    const [baseNav, setBaseNav] = useState();
    const [userNav, setUserNav] = useState();
    const [activeOrder, setActiveOrder] = useState();
    useEffect(() => {
        console.log(`Location: ${window.location.pathname}`)
        const pathNm = window.location.pathname;
        const baseNavItems = [];
        const setIfActive = (srchPath: string)=>{
            if (pathNm===srchPath || `${pathNm}/`===srchPath) {
                return('nav-item active');
            } else {
                return('nav-item');
            }
        };
        baseNavItems.push(
            <li className={setIfActive('/')} key="/">
                <Link className='nav-item nav-link' replace to='/'>Home</Link>
            </li>
        );
        baseNavItems.push(
            <li className={setIfActive('/orders/')} key="/orders">
                <Link className='nav-item nav-link' replace to='/orders/'>Orders</Link>
            </li>
        );
        baseNavItems.push(
            <li className="nav-item" key="reportIssue">
                <a className='nav-item nav-link'
                   href="https://github.com/cch71/T27FundraisingWebApp/issues">Report Issue</a>
            </li>
        );
        
        setBaseNav(baseNavItems);

        
        auth.getSession().then(([isValid, session])=>{
            if (isValid && session) {
                setUserNav(
                    <span className="navbar-nav nav-item dropdown">
                        <a className="nav-link dropdown-toggle" href="#" id="navbarDropdownMenuLink"
                           data-toggle="dropdown" aria-haspopup="false" aria-expanded="false">
                            {auth.currentUser().getUsername()}
                        </a>
                        <div className="dropdown-menu" aria-labelledby="navbarDropdownMenuLink">
                            <Link className='dropdown-item' replace to='/signon/'>Signout</Link>
                        </div>
                    </span>
                );
            }

            if (orderDb.getActiveOrder()) {
                setActiveOrder(
                    <li className={setIfActive('/order_step_1/')}>
                        <Link className='nav-item nav-link' replace to='/order_step_1/'>Open Order</Link>
                    </li>
                );
            }
        });
    }, []);


    return (
        <nav className="navbar navbar-expand-sm navbar-light bg-light">
            <a className="navbar-brand" href="#">
                <span>
                    <img className="navbar-logo mr-2" src={t27patch} alt="Logo" />
                    Fundraiser
                </span>
            </a>

            <button className="navbar-toggler" type="button" data-toggle="collapse" data-target="#navbarNav"
                    aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
                <span className="navbar-toggler-icon"></span>
            </button>

            <div className="collapse navbar-collapse" id="navbarNav">
                <ul className="navbar-nav mr-auto">
                    {baseNav}
                    {activeOrder}
                </ul>
                {userNav}
            </div>
        </nav>
    );
}

export default NavBar
