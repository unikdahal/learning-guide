# Remaining computation and learning topics: evidence record

Checked 17 November 2026. Scope authority: [NEC ACtE syllabus](https://nec.gov.np/uploads/Syllabus-Registration/1763716654201_Syllabus%20Computer%20Engineering.pdf), 6.2, 6.5, 9.3, 9.5 and 9.6. The new lessons give selected worked examples, not exhaustive coverage or past-year attributions.

## Verified traces

- Liang–Barsky: window [10,50]×[10,50], segment (−10,10)→(70,50). p=(−80,80,−40,40), q=(−20,60,0,40). Parameter interval [0.25,0.75] gives (10,20)→(50,40). A vertical segment at x=5 is rejected without division by zero.
- Bayesian chain A→B→C: P(A)=0.2, P(B|A)=0.8/0.1 and P(C|B)=0.7/0.2. Enumeration gives P(B)=0.24, P(C)=0.32, P(A,C)=0.12 and P(A|C)=0.375.
- Binary Naive Bayes example: unsmoothed scores 0.2 and 0.025 give posterior 8/9. Add-one smoothing with binary features gives scores 0.1875 and 0.044444… and posterior about 0.8085. This is a conditional-independence model, not a general guarantee about real data.
- Adaline update: x=(1,2,1), w=(0.2,0.5,−0.1), target 1 and step 0.1 give output 1.1, update (−0.01,−0.02,−0.01), new weights (0.19,0.48,−0.11), and new output 1.04.
- Three-unit Hopfield check: symmetric off-diagonal weights 1, diagonal/thresholds 0. Updating the first unit of (−1,+1,+1) produces (+1,+1,+1); energy changes from +1 to −3.

## Independent-review corrections

The final lessons incorporate the following corrections:

1. In the CFL proof, a substring touching both the a and c blocks in a^p b^p c^p needs at least p+2 symbols. The bounded-window argument is specific to the selected string, not arbitrary symbol groups.
2. D-separation guarantees independence for every compatible CPT. An active path permits dependence, but special CPTs can still make variables independent. An observed collider descendant also opens the path.
3. Fitness-proportional selection requires nonnegative fitness and a positive total. Negative scores and all-zero fitness need explicit treatment.
4. Deterministic gradient descent on a positive-definite expected quadratic has an eigenvalue-based convergence condition. This is not an unconditional sample-path guarantee for constant-step stochastic LMS; noise can leave residual fluctuations.
5. Orthogonality removes cross-pattern inner-product terms. Nonorthogonal patterns may introduce harmful crosstalk but do not universally prevent recall.
6. Hopfield convergence uses finite bipolar states, symmetric zero-diagonal weights, fair asynchronous updates and a keep-state rule for zero local fields. Every actual change strictly lowers energy. The final state need not be the intended memory.
7. Draft capacity-threshold claims were removed. No universal finite-network storage percentage or attribution MCQ is retained.

## Sources and limits

- [UC Berkeley CS188: d-separation](https://inst.eecs.berkeley.edu/~cs188/textbook/bayes-nets/d-separation.html) supports the distinction between graphical independence guarantees and possible dependence.
- Standard academic references named in the lessons include Sipser for formal languages; Hearn/Baker for clipping; Russell/Norvig for Bayesian networks; Widrow/Hoff for Adaline; and Hopfield's 1982 paper for energy-based recurrent networks. Naming a text is not a claim that every edition was fetched or every theorem was independently proved in this session.
- The independent reviewer checked both complete lessons and all 120 MCQ answer indices, numeric cases and explanations. Parent integration reviewed the listed corrections and replaced the capacity question with a checkable energy update.
- All question sets remain original practice. Exact PSC overlap is editorial and must be checked against the syllabus for the candidate's advertised post.
