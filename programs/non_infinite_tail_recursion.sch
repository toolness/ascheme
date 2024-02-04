; A silly function that just loops (recurses via tail calls) the
; given number of times and then returns 0.
(define (boop loop-times i)
  (if (< i loop-times)
    (boop loop-times (+ i 1))
   0)
)

(boop 300 0)
