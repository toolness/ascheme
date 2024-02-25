(define (double a) (+ a a))
(define (halve a) (assert (even? a)) (/ a 2))
(define (even? n) (= (remainder n 2) 0))

; invariant: 'a * b + z' should be unchanged from state to state.
(define (fast-mul-iterative-helper a b z)
  ;(print-and-eval b)
  (cond
    ((= b 0) z)
    ((= b 2) (+ (double a) z))
    ; This relies on the fact that `a * b = 2a * ½b`.
    ((even? b) (fast-mul-iterative-helper (double a) (halve b) z))
    (else (fast-mul-iterative-helper a (- b 1) (+ z a)))
  )
)

(define (fast-mul-iterative a b)
  (fast-mul-iterative-helper a b 0)
)

(test-eq (fast-mul-iterative 5 0) 0)
(test-eq (fast-mul-iterative 5 1) 5)
(test-eq (fast-mul-iterative 5 2) 10)
(test-eq (fast-mul-iterative 5 3) 15)
(test-eq (fast-mul-iterative 5 4) 20)
(test-eq (fast-mul-iterative 5 5) 25)
(test-eq (fast-mul-iterative 5 6) 30)
(test-eq (fast-mul-iterative 5 7) 35)
(test-eq (fast-mul-iterative 5 8) 40)
