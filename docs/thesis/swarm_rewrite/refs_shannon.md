# Bibliography: Swarm Shannon Limits & Cryptography Foundations

This bibliography documents the foundational publications and original papers underpinning the Swarm Shannon Limits, cryptographic structures, and cognitive bounds described in the thesis.

---

## 1. Shannon Entropy and Communication Limits

### Claude Shannon (1948)
*   **Title:** A Mathematical Theory of Communication
*   **Author:** Claude E. Shannon
*   **Year:** 1948
*   **Publication Venue:** *Bell System Technical Journal*, Vol. 27, No. 3, pp. 379–423 (July 1948) and Vol. 27, No. 4, pp. 623–656 (October 1948).
*   **DOI:** [10.1002/j.1538-7305.1948.tb01338.x](https://doi.org/10.1002/j.1538-7305.1948.tb01338.x)
*   **URL:** [https://doi.org/10.1002/j.1538-7305.1948.tb01338.x](https://doi.org/10.1002/j.1538-7305.1948.tb01338.x)
*   **BibTeX:**
    ```bibtex
    @article{shannon1948,
      author  = {Shannon, Claude E.},
      title   = {A Mathematical Theory of Communication},
      journal = {Bell System Technical Journal},
      volume  = {27},
      number  = {3},
      pages   = {379--423},
      year    = {1948},
      publisher = {Nokia Bell Labs}
    }
    ```
*   **Swarm Relevance:** Establishes the information-theoretic capacity and entropy bounds of communication channels. It dictates the limits of state transmission in the swarm, justifying the transition from raw execution copying to cryptographic causal receipt chains.

---

## 2. Merkle Mountain Ranges and Merkle Trees

### Ralph Merkle (1979) — Foundational Merkle Trees
*   **Title:** Secrecy, Authentication, and Public Key Systems
*   **Author:** Ralph C. Merkle
*   **Year:** 1979
*   **Publication Venue:** Ph.D. Dissertation, Stanford University (Technical Report No. 1979-1, Information Systems Laboratory).
*   **URL:** [https://www.merkle.com/papers/thesis1979.pdf](https://www.merkle.com/papers/thesis1979.pdf)
*   **BibTeX:**
    ```bibtex
    @phdthesis{merkle1979thesis,
      author      = {Merkle, Ralph C.},
      title       = {Secrecy, Authentication, and Public Key Systems},
      school      = {Stanford University},
      year        = {1979},
      type        = {{Ph.D.} Thesis},
      note        = {Technical Report No. 1979-1, Information Systems Laboratory}
    }
    ```
*   **Alternative Published Version:**
    *   **Title:** A Certified Digital Signature
    *   **Publication Venue:** *Advances in Cryptology — CRYPTO '89*, Lecture Notes in Computer Science, Vol. 435, Springer, Berlin, Heidelberg, pp. 218–238.
    *   **DOI:** [10.1007/0-387-34805-0_21](https://doi.org/10.1007/0-387-34805-0_21)
    *   **URL:** [https://link.springer.com/chapter/10.1007/0-387-34805-0_21](https://link.springer.com/chapter/10.1007/0-387-34805-0_21)
    *   **BibTeX:**
        ```bibtex
        @inproceedings{merkle1989certified,
          author    = {Merkle, Ralph C.},
          title     = {A Certified Digital Signature},
          booktitle = {Advances in Cryptology --- CRYPTO '89 Proceedings},
          series    = {Lecture Notes in Computer Science},
          volume    = {435},
          pages     = {218--238},
          publisher = {Springer},
          year      = {1989}
        }
        ```

### Peter Todd (2012) — Merkle Mountain Ranges (MMR)
*   **Title:** Merkle Mountain Ranges
*   **Author:** Peter Todd
*   **Year:** 2012
*   **Publication Venue:** Technical specification documentation in the OpenTimestamps project.
*   **URL:** [https://github.com/opentimestamps/opentimestamps-server/blob/master/doc/merkle-mountain-range.md](https://github.com/opentimestamps/opentimestamps-server/blob/master/doc/merkle-mountain-range.md)
*   **BibTeX:**
    ```bibtex
    @misc{todd2012mmr,
      author       = {Todd, Peter},
      title        = {Merkle Mountain Ranges},
      howpublished = {OpenTimestamps Documentation},
      year         = {2012},
      url          = {https://github.com/opentimestamps/opentimestamps-server/blob/master/doc/merkle-mountain-range.md}
    }
    ```
*   **Swarm Relevance:** Merkle trees and Merkle Mountain Ranges enable the commitment of high-dimensional swarm state vectors $S_t$ into low-entropy digests. MMRs specifically allow append-only historical proof trees that scale logarithmically, enabling verification of arbitrary historical segments.

---

## 3. Cryptographic Accumulators

### Josh Benaloh & Michael de Mare (1993)
*   **Title:** One-Way Accumulators: A Decentralized Alternative to Digital Signatures (Extended Abstract)
*   *Note: Frequently cited or variations referenced in secondary literature as "One-way accumulators: A decentralized alternative to double-signature schemes" due to its application as a decentralized double-signature alternative in timestamp and membership protocols.*
*   **Authors:** Josh Benaloh and Michael de Mare
*   **Year:** 1993 (Conference Presentation); 1994 (Book Publication)
*   **Publication Venue:** *Advances in Cryptology — EUROCRYPT '93*, Lecture Notes in Computer Science, Vol. 765, Springer, Berlin, Heidelberg, pp. 274–285.
*   **DOI:** [10.1007/3-540-48285-7_24](https://doi.org/10.1007/3-540-48285-7_24)
*   **URL:** [https://link.springer.com/chapter/10.1007/3-540-48285-7_24](https://link.springer.com/chapter/10.1007/3-540-48285-7_24)
*   **BibTeX:**
    ```bibtex
    @inproceedings{benaloh1993accumulators,
      author    = {Benaloh, Josh and de Mare, Michael},
      title     = {One-Way Accumulators: A Decentralized Alternative to Digital Signatures},
      booktitle = {Advances in Cryptology --- EUROCRYPT '93},
      series    = {Lecture Notes in Computer Science},
      volume    = {765},
      pages     = {274--285},
      publisher = {Springer},
      year      = {1993}
    }
    ```
*   **Swarm Relevance:** Introduces the concept of quasi-commutative cryptographic accumulators. In the swarm context, this enables decentralized and efficient membership proofs (witnesses) that establish whether an agent state belongs to the admitted subset without needing a centralized trusted coordinator.

---

## 4. Cognitive Limits ($\kappa$)

### George Miller (1956)
*   **Title:** The Magical Number Seven, Plus or Minus Two: Some Limits on our Capacity for Processing Information
*   **Author:** George A. Miller
*   **Year:** 1956
*   **Publication Venue:** *Psychological Review*, Vol. 63, No. 2, pp. 81–97.
*   **DOI:** [10.1037/h0043158](https://doi.org/10.1037/h0043158)
*   **URL:** [https://doi.org/10.1037/h0043158](https://doi.org/10.1037/h0043158)
*   **BibTeX:**
    ```bibtex
    @article{miller1956magical,
      author  = {Miller, George A.},
      title   = {The magical number seven, plus or minus two: Some limits on our capacity for processing information},
      journal = {Psychological Review},
      volume  = {63},
      number  = {2},
      pages   = {81--97},
      year    = {1956}
    }
    ```

### Nelson Cowan (2001)
*   **Title:** The magical number 4 in short-term memory: A reconsideration of mental storage capacity
*   **Author:** Nelson Cowan
*   **Year:** 2001
*   **Publication Venue:** *Behavioral and Brain Sciences*, Vol. 24, No. 1, pp. 87–114.
*   **DOI:** [10.1017/S0140525X01003922](https://doi.org/10.1017/S0140525X01003922)
*   **URL:** [https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/magical-number-4-in-shortterm-memory-a-reconsideration-of-mental-storage-capacity/F7CF6DF3F62985BE1F985EC1CB0D793C](https://www.cambridge.org/core/journals/behavioral-and-brain-sciences/article/magical-number-4-in-shortterm-memory-a-reconsideration-of-mental-storage-capacity/F7CF6DF3F62985BE1F985EC1CB0D793C)
*   **BibTeX:**
    ```bibtex
    @article{cowan2001magical,
      author  = {Cowan, Nelson},
      title   = {The magical number 4 in short-term memory: A reconsideration of mental storage capacity},
      journal = {Behavioral and Brain Sciences},
      volume  = {24},
      number  = {1},
      pages   = {87--114},
      year    = {2001}
    }
    ```
*   **Swarm Relevance:** Formalizes cognitive working memory bounds ($\kappa \approx 4$ chunks). This defines the mathematical limit of context capacity for human operators, defining the verification cost parameter $O(\kappa)$ that spans the comprehension-verification gap in the limit of swarm scale.