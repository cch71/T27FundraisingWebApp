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
        `gatsby-plugin-sharp`,
        `gatsby-transformer-sharp`,
        {
            resolve: `gatsby-plugin-manifest`,
            options: {
                name: `t27-mulch-order-system`,
                short_name: `t27Orders`,
                start_url: `/`,
                background_color: `#f7f0eb`,
                theme_color: `#a2466c`,
                display: `minimal-ui`,
                icon: `static/favicon.ico`,
            },
        },
        'gatsby-plugin-offline',
        {
            resolve: `gatsby-plugin-sass`,
            options: {
                implementation: require("sass"),
            },
        },
        'gatsby-plugin-ts'
    ],
}
