//gatsby-node.js

exports.onCreatePage = ({ page, actions }) => {
    const { createPage } = actions;
    if (page.path === `/`) {
        page.matchPath = `/*`;
        createPage(page);
    }

    if (page.path.match(/signon/)) {
        page.context.layout = 'signon';
        createPage(page)
    }

};

exports.onCreateWebpackConfig = ({
    actions,
}) => {
    const { setWebpackConfig } = actions;
    setWebpackConfig({
        externals: {
            jquery: 'jQuery', // important: 'Q' capitalized
        }
    })
}
