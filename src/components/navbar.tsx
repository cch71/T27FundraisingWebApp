import React from 'react'
import { Link, /*graphql, StaticQuery*/ } from 'gatsby'

const NavBar = () => {
    const toggleNavBar = () => {
        setActive(!active)
    }

    return (
        <nav className="navbar navbar-light bg-light navbar-expand-lg">
            <a className="navbar-brand" href="#">T27 Fundraiser</a>

            <button className="navbar-toggler" type="button" data-toggle="collapse"
                    data-target="#t27Navbar" aria-controls="t27Navbar"
                    aria-expanded="false" aria-label="Toggle navigation">
                <span className="navbar-toggler-icon"></span>
            </button>

            <div className="collapse navbar-collapse" id="t27Navbar">
                <div className="navbar-nav mr-auto">
                    <Link className='nav-item nav-link' replace to='/'>Home</Link>
                    <Link className='nav-item nav-link' replace to='/orders/'>Orders</Link>
                    <Link className='nav-item nav-link' replace to='/signon/'>SignOn</Link>
                </div>
            </div>
        </nav>
    );
}

export default NavBar
