import React, { useState } from 'react'
import {Nav, Navbar} from 'react-bootstrap';
import { Link, /*graphql, StaticQuery*/ } from 'gatsby'

const NavBar = () => {
    const [active, setActive] = useState(false)

    const toggleNavBar = () => {
        setActive(!active)
    }

    return (
        <Navbar bg="light" expand="lg">
            <Navbar.Brand href="#home">T27 Fundraiser</Navbar.Brand>
            <Navbar.Toggle aria-controls="basic-navbar-nav" />
            <Navbar.Collapse id="basic-navbar-nav">
                <Nav className="mr-auto">
                    <Link className='nav-item nav-link' replace to='/'>Home</Link>
                    <Link className='nav-item nav-link' replace to='/orders'>Orders</Link>
                    <Link className='nav-item nav-link' replace to='/signon'>SignOn</Link>
                </Nav>
            </Navbar.Collapse>
        </Navbar>
    );
}

export default NavBar
