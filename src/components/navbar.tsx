import React, { useState, useEffect } from 'react'
import { Link, navigate } from 'gatsby'
import auth from "../js/auth"
import t27patch from "../../static/t27patch.png"
import {orderDb} from "../js/ordersdb"
import {saveCurrentOrder} from "../js/utils"


const NavBar = (props) => {
	  const activePathNm = (typeof window !== 'undefined')?window.location.pathname:undefined;
    console.log(`Path Name ${activePathNm}`);

    const setIfActive = (srchPath: string) => {
        if (activePathNm===srchPath || `${activePathNm}/`===srchPath) {
            return('nav-item nav-link active');
        } else {
            return('nav-item nav-link');
        }
    };

    const collapseNav = () => {
        const srchPath = '/order_step_1/';
        if (activePathNm===srchPath || `${activePathNm}/`===srchPath) {
            saveCurrentOrder(); //If we navigate away lets save current order if it is active
        }
        jQuery(".navbar-collapse").collapse('hide');
    }

    const handleSignout = ()=>{
        collapseNav();
        auth.signOut();
        if ("/" === activePathNm) {
            window.location.reload(false);
        } else {
            navigate("/");
        }
    };


    const [userName, setUserName] = useState();
    useEffect(() => {
        const onAsyncView = async ()=>{
            //If this throws then we aren't authenticated so don't show bar anyways
            const [uid, userGroups] = await auth.getUserIdAndGroups();

            if (userGroups && userGroups.includes("FrAdmins")) {
				        //TODO: Placeholder for the Admin Menu Options
            }

            setUserName(uid);
        };

        onAsyncView()
            .then()
            .catch((err)=>{});
    }, []);

    return (
        <nav className="navbar sticky-top navbar-expand-sm navbar-light bg-light" id="primaryNavBar">
			      <a className="navbar-brand" href="#">
				        <span>
					          <img className="navbar-logo ms-2" src={t27patch} alt="Logo" />
				        </span>
			      </a>

			      <button className="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav"
				            aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
				        <span className="navbar-toggler-icon"></span>
			      </button>

			      <div className="collapse navbar-collapse" id="navbarNav">
				        <ul className="navbar-nav me-auto">
					          <li>
						            <Link className={setIfActive('/')} replace to='/' onClick={collapseNav}>Home</Link>
					          </li>
					          <li>
						            <Link className={setIfActive('/orders/')} replace to='/orders/' onClick={collapseNav}>
                            Reports
                        </Link>
					          </li>
					          <li style={{display: (orderDb.getActiveOrder()?'block':'none')}} >
						            <Link className={setIfActive('/order_step_1/')} replace to='/order_step_1/' onClick={collapseNav}>
                            Open Order
                        </Link>
					          </li>
					          <li>
						            <a className='nav-item nav-link' href='https://cch71.github.io/T27FundraisingWebApp/'
                           target="_blank" onClick={collapseNav}>
                            Help
                        </a>
					          </li>

				        </ul>
                <span className="navbar-nav nav-item dropdown">
                    <a className="nav-link dropdown-toggle" href="#" id="navbarDropdown"
                       data-bs-toggle="dropdown" aria-expanded="false" role="button">
                        {userName}
                    </a>
                    <div className="dropdown-menu dropdown-menu-end" aria-labelledby="navbarDropdown">
                        <a className='dropdown-item' href="#" onClick={handleSignout}>Signout</a>
                        <a className='dropdown-item'
                           href="https://github.com/cch71/T27FundraisingWebApp/issues"
                           onClick={collapseNav}>Report Issue</a>
                    </div>
                </span>
			      </div>
		    </nav>
    );
}

export default NavBar
