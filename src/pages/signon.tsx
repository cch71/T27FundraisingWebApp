import React, { useState } from "react"
import NavBar from "../components/navbar"
import {Card, Form, Button} from "react-bootstrap"
import auth from "../js/auth"
import { navigate } from "gatsby"

export default function signon() {
    const [validated, setValidated] = useState(false);

    const onFormSubmission = (event: any) => {
        const form = event.currentTarget;

        const loginId = form[0].value;
        const pw = form[1].value;
        console.log(`Form submttted uid: pw: ${loginId} ${pw}`)

        event.preventDefault();
        event.stopPropagation();

        const onSuccess=(autoInfo: any)=>{
            console.log(autoInfo);
            navigate('/', {replace: true});
        };
        const onFailure=()=>{
            console.error("authenticaiton Error");
            setValidated(false);
        };
        auth.signIn(loginId, pw, onSuccess, onFailure);
    };

    const curUsr = auth.currentUser();
    const loginId = (curUsr && curUsr.username) ? curUsr.username : '';


    return (
        <div className="col-xs-1 d-flex justify-content-center">
            <Card style={{ width: '20rem' }}>
                <Card.Body>
                    <Card.Title>Sign On</Card.Title>
                    <Form noValidate validated={validated} onSubmit={onFormSubmission}>
                        <Form.Group controlId="formLoginId">
                            <Form.Label>Login ID</Form.Label>
                            <Form.Control type="text" placeholder="Enter Login ID" defaultValue={loginId} />
                            <Form.Text className="text-muted">
                                If you don't know what yours is check with TODO: Email
                            </Form.Text>
                        </Form.Group>

                        <Form.Group controlId="formPassword">
                            <Form.Label>Password</Form.Label>
                            <Form.Control type="password" placeholder="Password" />
                        </Form.Group>
                        <Button variant="primary" type="submit">
                            Submit
                        </Button>
                    </Form>
                </Card.Body>
            </Card>
        </div>
    );
}
