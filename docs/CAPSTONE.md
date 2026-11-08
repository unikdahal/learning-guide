# Capstone capability and acceptance plan

The capstone is planned, not implemented. Running all 22 TPC-H queries is a final acceptance gate,
not an existing capability. The standard orientation fixture is simplified; it is not the eight-table
TPC-H dataset and cannot validate those queries.

## Milestones

1. **Core engine:** scan, typed expressions, filter/project, inner hash join, fixed-size aggregates,
   total ordering and limit. Start with a declared SQL subset and Q1/Q3/Q6 fixtures. Reject unsupported
   syntax explicitly. Compare exact bags or ordered rows with an independent engine before timing.
2. **Complete query suite:** implement the capabilities below, including subquery lowering and
   required scalar functions, then validate every query at the same dataset seed and scale. Keep
   parser/binder reuse explicit; using another engine to execute a query is not implementing that operator.
3. **Resource contract:** parallel execution, managed-memory limits, spilling, cancellation, and cleanup.
   Correct answers are the first gate; bounded resources and performance are separate gates.

Read Modules 01, 04–10 and 12 before the core work. Module 13 precedes the cost-model milestone;
Module 11 precedes memory/spilling. Introduce capabilities just in time rather than making every
expert chapter a prerequisite.

| Query | Required capabilities beyond basic scan/filter/project | Correctness cases |
| --- | --- | --- |
| Q1 | Decimal SUM/AVG/COUNT, date interval, multi-key GROUP BY, sort | Decimal scale, cutoff boundary |
| Q2 | Multiway joins, correlated scalar MIN, sorting and LIMIT | Equal minima, correlated scope |
| Q3 | Joins, date comparisons, aggregate expression, top-K | Ties, decimal arithmetic |
| Q4 | Correlated EXISTS, grouping | Multiple matches count once |
| Q5 | Multiway joins, date interval, grouped revenue | Join multiplicity |
| Q6 | Ungrouped SUM, decimal ranges and date filters | Empty/all-NULL aggregate |
| Q7 | Derived table, EXTRACT year, multiway joins | Year boundary and aliases |
| Q8 | CASE inside SUM, ratio, derived table | Conditional NULL/type rules |
| Q9 | LIKE, EXTRACT, arithmetic aggregate, derived joins | Duplicate-sensitive totals |
| Q10 | Multi-key grouping, join filters, top-K | Stable result ordering |
| Q11 | Scalar aggregate subquery in HAVING | Scalar cardinality and empty input |
| Q12 | CASE, COUNT/SUM, date/interval predicates | Conditional aggregate state |
| Q13 | LEFT JOIN with ON predicate, nested grouping | Unmatched rows and COUNT semantics |
| Q14 | CASE revenue ratio, LIKE, date interval | Decimal division and zero denominator |
| Q15 | CTE or view lowering, scalar MAX subquery | Multiple maximum suppliers |
| Q16 | NOT IN, COUNT DISTINCT, substring/LIKE filters | Inner NULLs, distinct state |
| Q17 | Correlated AVG and scalar aggregate | Correlation, empty matches |
| Q18 | IN over GROUP BY/HAVING subquery, multiway joins | Semi-join duplicates |
| Q19 | Compound OR, multiway predicate binding, decimal SUM | Predicate grouping and coercion |
| Q20 | IN, nested correlated SUM/EXISTS, date arithmetic | Multiple correlation scopes |
| Q21 | EXISTS and NOT EXISTS with multiple aliases, count/top-K | Distinct supplier matching |
| Q22 | SUBSTRING, scalar AVG, NOT EXISTS, grouping | String codes, outer NULLs |

Before claiming completion, add a versioned fixture manifest for each query: source SQL checksum,
data generator/version, scale, parameters, expected schema, expected rows and ordering, reference-engine
version, and comparison policy. Compare exact integers/decimals and NULL markers; any floating-point
tolerance must be stated per output column. Preserve duplicate multiplicities. Normalize ordering only
where the query does not promise it. Include small adversarial fixtures alongside benchmark-scale data.
Use an approved TPC-H data generator and respect its distribution terms; do not fabricate expected answers.
