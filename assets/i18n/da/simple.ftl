missing-arg-error = Fejl: Giv venligst et tal som indtastning.
input-parse-error = Error: Could not parse input `{ $input }`. Reason: { $reason }
response-msg =
    {$value ->
         [one] "{ $input }" har et Collatz step.
        *[other] "{ $input }" har { $value } Collatz steps.
    }
