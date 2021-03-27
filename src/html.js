import React from "react"
import PropTypes from "prop-types"

export default function HTML(props) {
    return (
        <html {...props.htmlAttributes}>
            <head>
                <meta charSet="utf-8" />
                <meta httpEquiv="x-ua-compatible" content="ie=edge" />
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1, shrink-to-fit=no"
                />

                <link rel="stylesheet" crossOrigin="anonymous"
                      href="https://cdn.jsdelivr.net/npm/bootstrap@5.0.0-beta3/dist/css/bootstrap.min.css"
                      integrity="sha384-eOJMYsd53ii+scO/bJGFsiCZc+5NDVN2yr8+0RDqr0Ql0h+rP48ckxlpbzKgwra6" />
                <link rel="stylesheet"
                      type="text/css"
                      href="https://cdn.datatables.net/v/bs4/dt-1.10.23/r-2.2.6/datatables.min.css" />
                <link rel="stylesheet"
                      href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.3.0/font/bootstrap-icons.css" />
                <link rel="stylesheet" type="text/css"
                      href="https://cdn.jsdelivr.net/npm/clocklet@0.3.0/css/clocklet.min.css" />

                <script src="https://code.jquery.com/jquery-3.5.1.min.js"
                        integrity="sha256-9/aliU8dGd2tb6OSsuzixeV4y/faTqgFtohetphbbj0="
                        crossOrigin="anonymous"></script>
                <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.0.0-beta3/dist/js/bootstrap.bundle.min.js"
                        integrity="sha384-JEW9xMcG8R+pH31jmWH6WWP0WintQrMb4s7ZOdauHnUtxwoG2vI5DkLtS3qm9Ekf"
                        crossOrigin="anonymous"></script>
                <script type="text/javascript"
                        src="https://cdn.datatables.net/v/bs4/dt-1.10.23/r-2.2.6/datatables.min.js"></script>
                <script type="text/javascript" src="https://www.gstatic.com/charts/loader.js" ></script>
                <script src="https://cdnjs.cloudflare.com/ajax/libs/PapaParse/5.3.0/papaparse.min.js"
                        integrity="sha512-rKFvwjvE4liWPlFnvH4ZhRDfNZ9FOpdkD/BU5gAIA3VS3vOQrQ5BjKgbO3kxebKhHdHcNUHLqxQYSoxee9UwgA=="
                        crossOrigin="anonymous"></script>
                <script type="text/javascript" crossOrigin="anonymous"
                        src="https://cdn.jsdelivr.net/npm/clocklet@0.3.0"></script>

                {props.headComponents}
            </head>
            <body {...props.bodyAttributes}>
                {props.preBodyComponents}
                <div
                    key={`body`}
                    id="___gatsby"
                    dangerouslySetInnerHTML={{ __html: props.body }}
                />
                {props.postBodyComponents}
            </body>
        </html>
    )
}

HTML.propTypes = {
    htmlAttributes: PropTypes.object,
    headComponents: PropTypes.array,
    bodyAttributes: PropTypes.object,
    preBodyComponents: PropTypes.array,
    body: PropTypes.string,
    postBodyComponents: PropTypes.array,
}
