

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
    }
    return undefined;
};

const removeRowWithTr = (dt, tr) => {
    const row = dt.row(tr);
    row.remove().draw();
};

export {getDataTable, removeRowWithTr}
