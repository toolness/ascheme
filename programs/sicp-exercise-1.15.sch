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

; As for part (b), it looks like the O(n) of space and number of steps is
; the same (the function is linear recursive) and appears to increase by
; 2 for every power of 10. I think this means that it's approximately O(log n).
; This is a bit surprising since the book doesn't seem to actually teach
; about O(log n) until the next section, and it's also non-obvious from the
; structure of the algorithm itself that this is its order of growth (it
; seems like it requires actually running the code and observing the pattern),
; but maybe I'm just doing it wrong.

; Also of note: the book hasn't actually taught `set!` or `display` or, as far
; as I can tell, any other mechanism that would allow students to actually
; track how many times `p` has been called. I'm not sure if this means that
; it's left up to the student to learn how to do that, or if the student is
; supposed to run the program in their head (which seems painful).
