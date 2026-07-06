# Swarm Verification & Reality Addressing: Foundational Bibliography

This bibliography compiles the foundational literature and specifications for Swarm Verification & Reality Addressing. It covers three main areas:
1. Language Server Protocol (LSP) & Language Server Index Format (LSIF)
2. Web Ontologies (OWL-Time, GeoSPARQL, PROV-O)
3. Causal Consistency & Graph Replay

---

## 1. Language Server Protocol (LSP) & LSIF

### Language Server Protocol (LSP) Specification
*   **Title:** Language Server Protocol Specification
*   **Author(s):** Microsoft Corporation (Originally designed by Dirk Bäumer and the Visual Studio Code team; standardized in collaboration with Red Hat, Codenvy, and the open-source community)
*   **Year:** 2016–Present (Originally announced in June 2016; v3.0.0 stable released in February 2017; v3.15.0 released in 2019)
*   **Publication Venue:** Microsoft GitHub / Open Specification Program
*   **URL:** [https://microsoft.github.io/language-server-protocol/](https://microsoft.github.io/language-server-protocol/)
*   **Annotation:** Defines a standard JSON-RPC protocol to exchange editor features (such as auto-complete, go-to-definition, and hover hints) between development tools and language servers. Used in Swarm Verification as a conceptual model for mapping virtual symbols to physical coordinates.

### Language Server Index Format (LSIF) Specification
*   **Title:** Language Server Index Format Specification
*   **Author(s):** Microsoft Corporation (With major implementation contributions and tooling support from Sourcegraph)
*   **Year:** 2019 (Announced by Dirk Bäumer in February 2019)
*   **Publication Venue:** Microsoft Language Server Protocol Project
*   **URL:** [https://microsoft.github.io/language-server-protocol/specifications/lsif/0.6.0/specification/](https://microsoft.github.io/language-server-protocol/specifications/lsif/0.6.0/specification/)
*   **Repository:** [https://github.com/microsoft/lsif-node](https://github.com/microsoft/lsif-node)
*   **Annotation:** A graph-based representation (vertices and edges) encoded in newline-delimited JSON (JSON Lines). It persists the semantic structure of a codebase to enable fast, serverless code intelligence. It is the direct precursor to SCIP and provides the topological blueprint for Reality Addressing.

### Source Code Intelligence Protocol (SCIP) (Successor to LSIF)
*   **Title:** SCIP: Source Code Intelligence Protocol
*   **Author(s):** Sourcegraph, Inc.
*   **Year:** 2022
*   **Publication Venue:** Sourcegraph GitHub Open Source Specifications
*   **URL:** [https://github.com/sourcegraph/scip](https://github.com/sourcegraph/scip)
*   **Annotation:** A Protobuf-based indexing format that succeeds LSIF. Designed specifically for massive monorepos to improve indexing performance, reduce index size, and facilitate multi-language navigation.

---

## 2. Web Ontologies

### W3C OWL-Time Specification
*   **Title:** Time Ontology in OWL
*   **Author(s):** Simon J. D. Cox and Chris Little (Editors of 2017 Recommendation); Jerry R. Hobbs and Feng Pan (Authors of original 2006 Working Group Note)
*   **Year:** 2017 (Recommendation version); 2006 (Original Working Group Note)
*   **Publication Venue:** World Wide Web Consortium (W3C) Recommendation / Open Geospatial Consortium (OGC) Standard
*   **Document ID / DOI:** OGC 16-071r3 (OGC Standard) / [10.62973/16-071r3](https://doi.org/10.62973/16-071r3)
*   **URLs:**
    *   2017 Recommendation: [https://www.w3.org/TR/owl-time/](https://www.w3.org/TR/owl-time/)
    *   2006 Note: [http://www.w3.org/TR/owl-time/](http://www.w3.org/TR/owl-time/)
*   **Annotation:** Establishes a formal temporal vocabulary for the Semantic Web. It defines concepts of instants, intervals, durations, and Allen's temporal relations. It is used in Swarm Verification for anchoring virtual states to temporal coordinates via `http://www.w3.org/2006/time#inXSDDateTimeStamp`.

### OGC GeoSPARQL Standard
*   **Title:** OGC GeoSPARQL – A Geographic Query Language for RDF Data
*   **Author(s):** Matthew Perry and John Herring (Editors)
*   **Year:** 2012 (v1.0); 2022 (v1.1, OGC 22-047)
*   **Publication Venue:** Open Geospatial Consortium (OGC) Implementation Standard
*   **Document ID:** OGC 11-052r4 (v1.0) / OGC 22-047 (v1.1)
*   **URL:** [https://www.opengis.net/doc/IS/geosparql/1.0](https://www.opengis.net/doc/IS/geosparql/1.0)
*   **Annotation:** Standardizes geospatial representations and topological query functions for RDF graph databases. In Reality Addressing, it provides the spatial dimension through Well-Known Text (WKT) literals via `http://www.opengis.net/ont/geosparql#asWKT`.

### W3C PROV-O (The PROV Ontology)
*   **Title:** PROV-O: The PROV Ontology
*   **Author(s):** Timothy Lebo, Satya Sahoo, and Deborah McGuinness (Lead Editors)
*   **Year:** 2013
*   **Publication Venue:** W3C Recommendation
*   **URL:** [https://www.w3.org/TR/2013/REC-prov-o-20130430/](https://www.w3.org/TR/2013/REC-prov-o-20130430/)
*   **Annotation:** Standardizes the representation of provenance histories for entities, activities, and agents. Used in Swarm Verification to attribute dynamic virtual state executions to physical or hardware agents using `http://www.w3.org/ns/prov#wasAttributedTo`.

---

## 3. Causal Consistency & Graph Replay

### Lamport's Logical Clocks
*   **Title:** Time, Clocks, and the Ordering of Events in a Distributed System
*   **Author(s):** Leslie Lamport
*   **Year:** 1978
*   **Publication Venue:** *Communications of the ACM* (CACM), Volume 21, Issue 7, Pages 558–565
*   **DOI:** [10.1145/359545.359563](https://doi.org/10.1145/359545.359563)
*   **URL:** [https://lamport.azurewebsites.net/pubs/time-clocks.pdf](https://lamport.azurewebsites.net/pubs/time-clocks.pdf)
*   **Annotation:** The foundational paper for ordering events in distributed systems without synchronized clocks. It introduces the partial order "happens-before" relation ($\to$) and logical clocks, providing the mathematical basis for state machine replication and causal consistency.

### Vector Clock Developments (Fidge)
*   **Title:** Timestamps in Message-Passing Systems That Preserve the Partial Ordering
*   **Author(s):** Colin J. Fidge
*   **Year:** 1988
*   **Publication Venue:** *Proceedings of the 11th Australian Computer Science Conference* (ACSC '88), Volume 10, Number 1, Pages 56–66
*   **URL:** [https://api.semanticscholar.org/CorpusID:18584970](https://api.semanticscholar.org/CorpusID:18584970)
*   **Annotation:** Independently proposed vector timestamps to characterize the partial ordering of events in message-passing systems, ensuring that two events are causally related if and only if their timestamps are ordered.

### Vector Clock Developments (Mattern)
*   **Title:** Virtual Time and Global States of Distributed Systems
*   **Author(s):** Friedemann Mattern
*   **Year:** 1989
*   **Publication Venue:** *Parallel and Distributed Algorithms* (Proceedings of the Workshop on Parallel and Distributed Algorithms, Chateau de Bonas, France, 1988), North-Holland Publishing, Pages 215–226
*   **URL:** [https://www.distrinet.inf.ethz.ch/publications/VirtTimeGlobStates.html](https://www.distrinet.inf.ethz.ch/publications/VirtTimeGlobStates.html)
*   **Annotation:** Formally analyzes vector clocks, virtual time, and consistent global states (cuts) in distributed environments, establishing the foundation for checking causal consistency in distributed graph replay geometries.

---

## Appendix: BibTeX Entries

For LaTeX integration within the master thesis bibliography (`refs.bib`), the following entries can be appended:

```bibtex
@misc{lsp-spec,
  author       = {{Microsoft Corporation}},
  title        = {Language Server Protocol Specification},
  howpublished = {\url{https://microsoft.github.io/language-server-protocol/}},
  year         = {2016},
  note         = {Accessed: 2026-07-04}
}

@misc{lsif-spec,
  author       = {{Microsoft Corporation}},
  title        = {Language Server Index Format ({LSIF}) Specification},
  version      = {0.6.0},
  howpublished = {\url{https://microsoft.github.io/language-server-protocol/specifications/lsif/0.6.0/specification/}},
  year         = {2019},
  note         = {Accessed: 2026-07-04}
}

@misc{scip-spec,
  author       = {{Sourcegraph, Inc.}},
  title        = {{SCIP}: Source Code Intelligence Protocol},
  howpublished = {\url{https://github.com/sourcegraph/scip}},
  year         = {2022},
  note         = {Accessed: 2026-07-04}
}

@techreport{w3c-owl-time,
  author       = {Simon J. D. Cox and Chris Little},
  title        = {Time Ontology in {OWL}},
  institution  = {World Wide Web Consortium (W3C)},
  type         = {{W3C Recommendation}},
  url          = {https://www.w3.org/TR/owl-time/},
  year         = {2017},
  note         = {Original Working Group Note by Jerry R. Hobbs and Feng Pan (2006)}
}

@techreport{ogc-geosparql,
  author       = {Matthew Perry and John Herring},
  title        = {{OGC} {GeoSPARQL} -- A Geographic Query Language for {RDF} Data},
  institution  = {Open Geospatial Consortium (OGC)},
  type         = {{OGC Implementation Standard}},
  number       = {OGC 11-052r4},
  url          = {https://www.opengis.net/doc/IS/geosparql/1.0},
  year         = {2012}
}

@techreport{w3c-prov-o,
  author       = {Timothy Lebo and Satya Sahoo and Deborah McGuinness},
  title        = {{PROV-O}: The {PROV} Ontology},
  institution  = {World Wide Web Consortium (W3C)},
  type         = {{W3C Recommendation}},
  url          = {https://www.w3.org/TR/2013/REC-prov-o-20130430/},
  year         = {2013}
}

@article{lamport1978time,
  author    = {Lamport, Leslie},
  title     = {Time, clocks, and the ordering of events in a distributed system},
  journal   = {Communications of the ACM},
  volume    = {21},
  number    = {7},
  pages     = {558--565},
  year      = {1978},
  doi       = {10.1145/359545.359563},
  url       = {https://lamport.azurewebsites.net/pubs/time-clocks.pdf}
}

@inproceedings{fidge1988timestamps,
  author    = {Fidge, Colin J.},
  title     = {Timestamps in Message-Passing Systems That Preserve the Partial Ordering},
  booktitle = {Proceedings of the 11th Australian Computer Science Conference (ACSC '88)},
  volume    = {10},
  number    = {1},
  pages     = {56--66},
  year      = {1988},
  url       = {https://api.semanticscholar.org/CorpusID:18584970}
}

@inproceedings{mattern1989virtual,
  author    = {Mattern, Friedemann},
  title     = {Virtual Time and Global States of Distributed Systems},
  booktitle = {Parallel and Distributed Algorithms},
  pages     = {215--226},
  year      = {1989},
  publisher = {North-Holland},
  url       = {https://www.distrinet.inf.ethz.ch/publications/VirtTimeGlobStates.html}
}