import React, { useState, useEffect } from 'react'
import { Link } from 'gatsby'
import auth from "../js/auth"

const NavBar = () => {
    const [userNav, setUserNav] = useState();
    useEffect(() => {
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
        });
    }, []);


    return (
        <nav className="navbar navbar-expand-lg navbar-light bg-light">
            <a className="navbar-brand" href="#">T27 Fundraiser</a>

            <button className="navbar-toggler" type="button" data-toggle="collapse" data-target="#navbarNav"
                    aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
                <span className="navbar-toggler-icon"></span>
            </button>

            <div className="collapse navbar-collapse" id="navbarNav">
                <ul className="navbar-nav mr-auto">
                    <li className="nav-item">
                        <Link className='nav-item nav-link' replace to='/'>Home</Link>
                    </li>
                    <li className="nav-item">
                        <Link className='nav-item nav-link' replace to='/orders/'>Orders</Link>
                    </li>
                    <li className="nav-item">
                        <Link className='nav-item nav-link' replace to='/signon/'>Signout</Link>
                    </li>
                </ul>
                {userNav}
            </div>
        </nav>
    );
}

export default NavBar
