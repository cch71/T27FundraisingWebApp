import React, { useState, useEffect } from "react";
import Helmet from 'react-helmet'
import NavBar from "../components/navbar"
import auth from "../js/auth"

const Layout = (props) => {
	  return (
		    <>
			      <Helmet title="T27 Fundraiser App"></Helmet>
					  <>
						    <NavBar />
						    {props.children}
					  </>
		    </>
	  );
}

// <Footer copyright={config.copyright} />

export default Layout
