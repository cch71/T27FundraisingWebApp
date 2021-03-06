/**
 * Configure your Gatsby site with this file.
 *
 * See: https://www.gatsbyjs.com/docs/gatsby-config/
 */
const path = require("path");


module.exports = {
    siteMetadata: {
        title: 'T27 Orders',
        description: 'Troop 27 Fundraising Order Form',
    },
    plugins: [
        {
            resolve: `gatsby-plugin-manifest`,
            options: {
                name: `T27 Fundraiser App`,
                short_name: `t27Orders`,
                start_url: `/`,
                background_color: `#f7f0eb`,
                theme_color: `#a2466c`,
                display: `standalone`,
                icon: `static/t27patch.png`,
            },
        },{
			resolve: `gatsby-plugin-layout`,
			options: {
				component: require.resolve(`./src/layouts/primary_layout.tsx`),
			},
		},
        `gatsby-plugin-sharp`,
        `gatsby-transformer-sharp`,
        `gatsby-plugin-offline`,
        {
            resolve: `gatsby-plugin-sass`,
            options: {
                implementation: require("sass"),
            },
        }
    ]
}
