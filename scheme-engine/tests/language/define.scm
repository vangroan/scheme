;; ===========
;; Definitions
;; ===========

(define x (+ 1 2))
(display x)
(assert (= x 3))

;; Global variables can be redefined
(define x 42)
(display x)
(assert (= x 42))

(define y #f)
(display y)

;; Basic local variable usage
(lambda (x y) (define z 3) (+ x y z))
