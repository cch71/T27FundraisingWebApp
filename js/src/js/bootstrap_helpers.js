function modalOp(id, op) {
    //const modal = bootstrap.Modal.getOrCreateInstance(`#${id}`);
    //modal.modal(op);
    $(`#${id}`).modal(op);
}

export {modalOp};
