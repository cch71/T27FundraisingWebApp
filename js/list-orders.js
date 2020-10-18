/*global WildRydes _config AmazonCognitoIdentity AWSCognito*/

var WildRydes = window.WildRydes || {};

(function createOrderScopeWrapper($) {
    /*
     * Cognito User Pool functions
     */

    var authToken;
    WildRydes.authToken.then(function setAuthToken(token) {
        if (token) {
            authToken = token;
        } else {
            window.location.href = '/signin.html';
        }
    }).catch(function handleTokenError(error) {
        alert(error);
        window.location.href = '/signin.html';
    });


    function populateTable(results) {
        //$('#order-list').submit(handleCreateOrder);
        console.log(results.items);

        var table = new Tabulator("#order-list", {
 	          height:205, // set height of table (in CSS or here), this enables the Virtual DOM and improves render speed dramatically (can be any valid css height value)
 	          data:results.items, //assign data to table
 	          layout:"fitColumns", //fit columns to width of table (optional)
 	          columns:[ //Define Table Columns
	 	            {title:"Scout", field:"OrderOwner", width:150},
	 	            {title:"Customer", field:"CustomerName"},
	 	            {title:"Address", field:"Address"},
	 	            {title:"Bags", field:"BagsPurchased"},
	 	            {title:"Spreading", field:"BagsToSpread"}
 	          ],
 	          rowClick:function(e, row){ //trigger an alert message when the row is clicked
 		            alert("Row " + row.getData().Address + " Clicked!!!!");
 	          },
        });
    }

    /*
     *  Event Handlers
     */

    $(function onDocReady() {
        $.ajax({
            method: 'POST',
            url: _config.api.invokeUrl + '/listorders',
            headers: {
                Authorization: authToken
            },
            data: JSON.stringify({}),
            contentType: 'application/json',
            success: populateTable,
            error: function ajaxError(jqXHR, textStatus, errorThrown) {
                console.error('Error requesting order list: ', textStatus, ', Details: ', errorThrown);
                console.error('Response: ', jqXHR.responseText);
                alert("Error requesting order list" + jqXHR.responseText);
            }
        });
    });


}(jQuery));
