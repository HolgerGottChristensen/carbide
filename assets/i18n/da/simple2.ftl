woo-response-msg = Er det godt? Jepper. Crazy. Simpelt
    { NUMBER($number,
        useGrouping: "min2",
        minimumIntegerDigits: 6,
        minimumFractionDigits: 10,
    )}
woo-response-msg2 = Er det godt? Jepper. Crazy. Simpelt
    { NUMBER($number,
        roundingMode: "halfEven",
        maximumFractionDigits: 1,
    )}

woo-response-msg3 = Tid default: { $time }

woo-response-msg4 = Dato og tid fuld: { DATETIME($time, dateStyle: "full", timeStyle: "full", timezoneStyle: "extended") }
woo-response-msg5 = Tid kort: { DATETIME($time, dateStyle: "hidden", timeStyle: "short") }