import React from 'react'
import { Link, /*graphql, StaticQuery*/ } from 'gatsby'

const NavBar = () => {
    const toggleNavBar = () => {
        setActive(!active)
    }

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
                        <Link className='nav-item nav-link' replace to='/signon/'>SignOn</Link>
                    </li>
                </ul>
            </div>
        </nav>
    );
}

export default NavBar
