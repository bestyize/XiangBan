"""Exact, streaming totals for one CSV column; no third-party dependencies."""
import argparse
import csv
import json
from decimal import Decimal, InvalidOperation, localcontext

def summarize(path, column, delimiter=","):
    total = Decimal("0")
    rows = 0
    blank = 0
    with open(path, encoding="utf-8-sig", newline="") as stream:
        reader = csv.DictReader(stream, delimiter=delimiter)
        if column not in (reader.fieldnames or []):
            raise ValueError(f"Column {column!r} is missing; available: {reader.fieldnames}")
        for row in reader:
            rows += 1
            value = row.get(column)
            if value is None:
                raise ValueError(f"Missing cell in record {rows}")
            if not value.strip():
                blank += 1
                continue
            try:
                number = Decimal(value.strip())
            except InvalidOperation as exc:
                raise ValueError(f"Invalid decimal in record {rows}: {value!r}") from exc
            if not number.is_finite():
                raise ValueError(f"Non-finite value in record {rows}")
            with localcontext() as context:
                exponent = min(total.as_tuple().exponent, number.as_tuple().exponent)
                precision = max(total.adjusted(), number.adjusted()) - exponent + 3
                if precision > 10_000:
                    raise ValueError(f"Numeric precision budget exceeded in record {rows}")
                context.prec = max(context.prec, precision)
                total += number
    return {"column": column, "records": rows, "blank_values": blank, "sum": str(total), "coverage": "all_records"}

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path")
    parser.add_argument("--column", required=True)
    parser.add_argument("--delimiter", default=",")
    args = parser.parse_args()
    print(json.dumps(summarize(args.path, args.column, args.delimiter), ensure_ascii=False))

