
;; ============
;; Conditionals
;; ============

;; if

(assert (= (if #f 1 2) 2))
(assert (= (if #t 1 2) 1))
(assert (if 0 #t #f))
(assert (if 1 #t #f))
(assert (if 3 #t #f))
(assert (= (if #t (+ 3 7) (* 3 7)) 10))
(assert (= (if #f (+ 3 7) (* 3 7)) 21))
(assert (if (= 3 3 3 3 3 3) #t #f))

;; cond
(assert-eq (cond (#t 1) (#f 2) (#f 3)) 1)
(assert-eq (cond (#f 1) (#t 2) (#t 3)) 2)
;; (assert-eq (cond (#f 1) (#f 2) (#f 3)) 3) ;; TODO: void literal
(assert-eq (cond ((< 0 1) 4) ((> 0 1) 5)) 4)
(assert-eq (cond (#f 4) (else 5)) 5)

;; (define choose (lambda (n) (cond ((< n 0) 1) ((= n 0) 2) ((> n 0) 3))))
;; (assert (= (choose 0) 2))
;;
;; (define iter (lambda (n)
;;   (cond
;;     ((>= n 5) n)           ; base
;;      (else (iter (+ n 1))) ; recursive
;;   )))
