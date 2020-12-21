import React, { useState, useEffect } from 'react'
import { Link } from 'gatsby'
import auth from "../js/auth"
import t27patch from "../../static/t27patch.png"
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
            <li key="AppHelp">
                <a className='nav-item nav-link'
                   href='https://cch71.github.io/T27FundraisingWebApp/' target="_blank">Help</a>
            </li>
        );

        setBaseNav(baseNavItems);


        auth.getUserIdAndGroups().then(([userName, userGroups])=>{
            setUserNav(
                <span className="navbar-nav nav-item dropdown">
                    <a className="nav-link dropdown-toggle" href="#" id="navbarDropdownMenuLink"
                       data-bs-toggle="dropdown" aria-haspopup="false" aria-expanded="false">
                        {userName}
                    </a>
                    <div className="dropdown-menu dropdown-menu-end" aria-labelledby="navbarDropdownMenuLink">
                        <Link className='dropdown-item' replace to='/signon/'>Signout</Link>
                        <a className='dropdown-item'
                           href="https://github.com/cch71/T27FundraisingWebApp/issues">Report Issue</a>
                    </div>
                </span>
            );
            if (userGroups && userGroups.includes("FrAdmins")) {
                console.log("This user is an admin");
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
                    <img className="navbar-logo me-2" src={t27patch} alt="Logo" />
                </span>
            </a>

            <button className="navbar-toggler" type="button" data-toggle="collapse" data-target="#navbarNav"
                    aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
                <span className="navbar-toggler-icon"></span>
            </button>

            <div className="collapse navbar-collapse" id="navbarNav">
                <ul className="navbar-nav me-auto">
                    {baseNav}
                    {activeOrder}
                </ul>
                {userNav}
            </div>
        </nav>
    );
}

export default NavBar
