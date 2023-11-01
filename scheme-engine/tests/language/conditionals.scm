
;; ============
;; Conditionals
;; ============

(assert (= (if #f 1 2) 2))
(assert (= (if #t 1 2) 1))
(assert (if 0 #t #f))
(assert (if 1 #t #f))
(assert (if 3 #t #f))
(assert (= (if #t (+ 3 7) (* 3 7)) 10))
(assert (= (if #f (+ 3 7) (* 3 7)) 21))
(assert (if (= 3 3 3 3 3 3) #t #f))
