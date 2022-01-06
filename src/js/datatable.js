
const getQuickViewReportDataTable = (params) => {
    const tableColumns = [
        {
            name: "OrderId",
            className: "all",
            visible: false
        },
        {
            title: "Name",
            className: "all"
        },
        {
            title: "Delivery Date",
            name: "DeliveryDate"
        }
    ];

    if (params.isMulchOrder) {
        tableColumns.push({
                          title: "Spreaders",
                          name: "Spreaders",
                          visible: false
        });
        tableColumns.push({
                          title: "Spreading",
                          className: "all",
                          render: (data, _, row, meta) => {
                              if (0!==row[meta.col-1].length) {
                                  return `${data}: Spread`
                              } else {
                                  return data;
                              }
                          }
        });
    }

    tableColumns.push({
                      title: "Order Owner",
                      name: "OrderOwner",
                      visible: params.showOrderOwner
    });

    tableColumns.push({
                      title: "Actions",
                      "orderable": false,
                      className: "all"
    });

    return new DataTable(
        params.id,
        {    
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "pdf", "print", 'colvis'
            ],
            responsive: true,
            deferRender: true,
            language: {
                paginate: {
                    previous: "<<",
                    next: ">>"
                }
            },
            columns: tableColumns
        }
    );
};


const getFullViewReportDataTable = (params) => {
    console.log("Setting Full Report View");
    let tableColumns = [
        { name: "OrderId", className: "all", visible: false },
        { title: "Name", className: "all" },
        { title: "Phone" },
        { title: "Email" },
        { title: "Address 1" },
        { title: "Address 2" },
        { title: "Neighborhood" },
        { title: "Delivery Date" }
    ];

    tableColumns.push({
                      title: "Spreaders",
                      name: "Spreaders",
                      visible: false
    });
    tableColumns.push({
                      title: "Spreading",
                      render: (data, _, row, meta) => {
                          if (0!==row[meta.col-1].length) {
                              return `${data}: Spread`
                          } else {
                              return data;
                          }
                      }
    });
    tableColumns.push({ title: "Bags" });

    tableColumns = tableColumns.concat([
        { title: "Special Instructions" },
        { title: "Verified" },
        { title: "Money Collected" },
        { title: "Donations" },
        { title: "Cash" },
        { title: "Check" },
        { title: "Check Numbers" },
        { title: "Total Amount" },
    ]);

    tableColumns.push({
                      title: "Order Owner",
                      name: "OrderOwner",
                      visible: params.showOrderOwner
    });

    tableColumns.push({
                      title: "Actions",
                      "orderable": false,
                      className: "all"
    });

    return new DataTable(
        params.id,
        {    
            dom: 'Bfrtip', //https://datatables.net/reference/option/dom
            buttons: [
                "csv", "copy", "excel", "pdf", "print", 'colvis'
            ],
            responsive: true,
            deferRender: true,
            language: {
                paginate: {
                    previous: "<<",
                    next: ">>"
                }
            },
            columns: tableColumns
        }
    );
};


const getDataTable = (params) => {
    if (params.reportType === "quick") {
        return getQuickViewReportDataTable(params);
    } else if (params.reportType === "full") {
        return getFullViewReportDataTable(params);
    }
    return undefined;
};

const removeRowWithTr = (dt, tr) => {
    const row = dt.row(tr);
    row.remove().draw();
};

export {getDataTable, removeRowWithTr}
