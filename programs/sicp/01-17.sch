(define (* a b)
  (if (= b 0)
      0
      (+ a (* a (- b 1)))
  )
)

(test-eq (* 5 0) 0)
(test-eq (* 0 5) 0)   ; This is funny because we are adding 0 five times.
(test-eq (* 5 1) 5)
(test-eq (* 5 2) 10)
(test-eq (* 5 5) 25)

(define (double a) (+ a a))
(define (halve a) (assert (even? a)) (/ a 2))
(define (even? n) (= (remainder n 2) 0))

(test-eq (double 12) 24)
(test-eq (halve 12) 6)

(define (fast-mul a b)
  ;(print-and-eval b)
  (cond
    ((= b 0) 0)
    ((= b 2) (double a))
    ((even? b) (fast-mul (double a) (halve b)))
    (else (+ a (fast-mul a (- b 1))))
  )
)

(test-eq (fast-mul 5 0) 0)
(test-eq (fast-mul 5 1) 5)
(test-eq (fast-mul 5 2) 10)
(test-eq (fast-mul 5 3) 15)
(test-eq (fast-mul 5 4) 20)
(test-eq (fast-mul 5 5) 25)
(test-eq (fast-mul 5 6) 30)
(test-eq (fast-mul 5 7) 35)
(test-eq (fast-mul 5 8) 40)
