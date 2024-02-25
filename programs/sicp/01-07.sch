(define (sqrt-iter guess x good-enough?)
  (if (good-enough? guess x)
    guess
    (sqrt-iter (improve guess x) x good-enough?)
  )
)

(define (terrible-good-enough? guess x) #t)

; This is the good-enough defined in SICP 1.1.7.
(define (original-good-enough? guess x)
  ;(print-and-eval guess)
  ;(print-and-eval x)
  (< (abs (- (square guess) x)) 0.001))

(define (improved-good-enough? guess x)
  (define delta (abs (- (square guess) x)))
  (define delta-as-fraction-of-x (/ delta x))
  ;(print-and-eval x)
  ;(print-and-eval guess)
  ;(print-and-eval delta)
  ;(print-and-eval delta-as-fraction-of-x)
  (< delta-as-fraction-of-x 0.0001)
)

(define (square x) (* x x))

(define (improve guess x)
  (average guess (/ x guess)))

(define (average x y)
  (/ (+ x y) 2))

(define (sqrt x)
  (sqrt-iter 1.0 x improved-good-enough?))

(print-and-eval (sqrt 9))
(print-and-eval (square (sqrt 1000)))
; This is actually 0.01, but with original-good-enough it returns 0.03.
(print-and-eval (sqrt 0.0001))
; This loops infinitely with original-good-enough. I don't think that
; f64 can actually hold these many digits of precision, as it shows up
; as 123456789123456780000000000000000000.
(print-and-eval (sqrt 123456789123456789123456789123456789))
