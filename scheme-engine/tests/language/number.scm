
(assert (= (- 3 2) 1))

(assert (= (+ 1 2 3 4 5 (- 6 7 8)) 6))

;; These need to move to boolean.scm
(assert (not (and 1 2 3 #f 5 6)))
