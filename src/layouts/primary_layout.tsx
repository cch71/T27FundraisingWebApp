import React, { useState, useEffect } from "react";
import Helmet from 'react-helmet'
import NavBar from "../components/navbar"

const Layout = (props) => {

	return (
		<>
			<Helmet title="T27 Fundraiser App"></Helmet>
			{
				(props.pageContext.layout === "signon")? (
					<>{props.children}</>
				) : (
					<>
						<NavBar />
						<>{props.children}</>
					</>
				)
			}
		</>
	);
}

// <Footer copyright={config.copyright} />

export default Layout
