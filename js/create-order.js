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

    function createOrder(params, onSuccess, onFailure) {
        $.ajax({
            method: 'POST',
            url: _config.api.invokeUrl + '/createorder',
            headers: {
                Authorization: authToken
            },
            data: JSON.stringify(params),
            contentType: 'application/json',
            success: onSuccess,
            error: function ajaxError(jqXHR, textStatus, errorThrown) {
                console.error('Error requesting ride: ', textStatus, ', Details: ', errorThrown);
                console.error('Response: ', jqXHR.responseText);
                onFailure(jqXHR.responseText);
            }
        });
    }


    /*
     *  Event Handlers
     */

    $(function onDocReady() {
        $('#createOrderForm').submit(handleCreateOrder);
    });


    function handleCreateOrder(event) {
        event.preventDefault();

        const params = {
            order_owner: $('#scout').val(),
            customer_name: $('#customerName').val(),
            address: $('#address').val(),
            bags: $('#bags').val(),
            bags_to_spread: $('#bagsToSpread').val()
        };

        event.preventDefault();
        createOrder(params,
            function onSuccess(result) {
                alert('Order Accepted\n' + result);
            },
            function onError(err) {
                alert(err);
            }
        );
    }
}(jQuery));
