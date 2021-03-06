//gatsby-node.js

exports.onCreatePage = ({ page, actions }) => {
    const { createPage } = actions;
    if (page.path === `/`) {
        page.matchPath = `/*`;
        createPage(page);
    }
};

exports.onCreateWebpackConfig = ({ actions }) => {
  actions.setWebpackConfig({
    resolve: {
       alias: {
          crypto: false
       }
    }
  })
}
