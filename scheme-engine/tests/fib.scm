(define (fib n) 
  (if (<= n 1)
    n 
    (+ (fib (- n 1)) (fib (- n 2))) 
  ))

(fib 0)
;> 0
(fib 1)
;> 1
(fib 5)
;> 5
(fib 12)
;> 144
