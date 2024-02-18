(define (cube x) (* x x x))

(define times-p-called 0)

(define (p x) 
  ;(print-and-eval x)
  (set! times-p-called (+ times-p-called 1))
  (- (* 3 x) (* 4 (cube x)))
)

(define (sine angle)
  (if (not (> (abs angle) 0.1))    ; Why didn't they use '<='?
      angle
      (p (sine (/ angle 3.0)))
  )
)

(define (sine-count-p angle)
  (set! times-p-called 0)
  (sine angle)
  times-p-called
)

(define pi 3.141592653)

(print-and-eval (sine 0))

(print-and-eval (sine (/ pi 2)))

(print-and-eval (sine (/ pi 4)))

(print-and-eval (sine 12.15))

(print-and-eval (sine-count-p 1))
(print-and-eval (sine-count-p 10))

; This is the answer to part (a), it's 5.
(print-and-eval (sine-count-p 12.15))

(print-and-eval (sine-count-p 100))
(print-and-eval (sine-count-p 1000))
(print-and-eval (sine-count-p 10000))
(print-and-eval (sine-count-p 100000))
(print-and-eval (sine-count-p 1000000))

; as for part (b), it looks like the O(n) of space and number of steps is
; the same (the function is not tail recursive) and appears to increase by
; 2 for every power of 10. I think this means that it's approximately O(log n).
