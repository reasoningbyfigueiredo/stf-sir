\documentclass[11pt]{article}

\usepackage{amsmath, amssymb, amsthm}
\usepackage{hyperref}
\usepackage{geometry}
\geometry{margin=1in}

\title{Coherence as a Structural Invariant in Semantic Systems}
\author{
Rogerio Figueiredo \
Independent Researcher \
\texttt{reasoningbyfigueiredo}
}
\date{2026}

\newtheorem{definition}{Definition}
\newtheorem{theorem}{Theorem}
\newtheorem{proposition}{Proposition}

\begin{document}

\maketitle

\begin{abstract}
We present a formal computational framework in which coherence is treated as a structural invariant of semantic systems. We define three layers of coherence—logical, computational, and operational—and show that information becomes useful only when it is coherent, grounded, and executable within a theory. We introduce a formal bridge between artifact-level representations and theory-level reasoning, define grounding as a verifiable structural relation, and establish a taxonomy of errors including contradiction, anomaly, and hallucination. The framework is instantiated in the STF-SIR architecture, where coherence becomes an auditable property of semantic compilation and reasoning systems.
\end{abstract}

\section{Introduction}

Semantic systems typically treat meaning as emergent. In contrast, we propose that coherence is a primary structural property of internal representations.

\textbf{Claim:} Meaning is not inferred post hoc, but constructed and validated through coherence-preserving transformations.

\section{System Model}

Let $\mathcal{U}$ be a universe of propositions.

\begin{definition}[Theory]
A theory $T \subseteq \mathcal{U}$ is a structured set of statements.
\end{definition}

\begin{definition}[Formula]
A formula is a syntactic representation over atoms using logical operators such as negation and implication.
\end{definition}

\section{Coherence}

\begin{definition}[Coherence]
The coherence of a theory $T$ is defined as:
[
\mathrm{Coh}(T) = (C_l, C_c, C_o)
]
\end{definition}

\subsection{Logical Coherence}

\begin{definition}
[
C_l(T) = 1 \iff T \not\models \bot
]
\end{definition}

\subsection{Computational Coherence}

\begin{definition}
[
C_c(T, B) = 1 \iff \exists V_B \text{ such that } V_B(T) \text{ verifies consistency within } B
]
\end{definition}

\subsection{Operational Coherence}

\begin{definition}
[
C_o(T) = 1 \iff \exists p \in \mathcal{U} \text{ such that } T \vdash p \text{ and } p \notin \mathrm{Cn}(\emptyset)
]
\end{definition}

\section{Information}

\begin{definition}[Admissible Information]
[
\mathrm{Adm}(m, T, A) = 1 \iff C_l(T \cup {m}) = 1 \wedge \mathrm{Ground}(m, A) = 1
]
\end{definition}

\begin{definition}[Executable Information]
[
\mathrm{Exec}(m, T) = 1 \iff C_o(T \cup {m}) = 1
]
\end{definition}

\begin{definition}[Useful Information]
[
\mathrm{Useful}(m, T, A) = 1 \iff \mathrm{Adm}(m, T, A) = 1 \wedge \mathrm{Exec}(m, T) = 1
]
\end{definition}

\begin{theorem}[Characterization of Useful Information]
A message is useful if and only if it is coherent, grounded, and operationally productive.
\end{theorem}

\begin{proof}
From the definitions:
[
\mathrm{Useful}(m, T, A) = 1
\iff
C_l(T \cup {m}) = 1 \wedge \mathrm{Ground}(m, A) = 1 \wedge C_o(T \cup {m}) = 1
]
which follows directly from the definitions of admissibility and executability.
\end{proof}

\section{Artifact-Theory Bridge}

Let $A$ be an artifact and $T$ a theory.

\begin{definition}[Bridge]
[
\beta : A \rightarrow T
]
maps semantic artifacts to theory-level statements.
\end{definition}

\begin{proposition}[Identity Preservation]
[
z_1 \neq z_2 \Rightarrow \beta(z_1) \neq \beta(z_2)
]
\end{proposition}

\begin{proposition}[Grounding Preservation]
[
\mathrm{Ground}(\beta(z)) = 1 \iff \mathrm{Ground}(z) = 1
]
\end{proposition}

\section{Grounding}

\begin{definition}
A statement $s$ is grounded if:
[
\exists a \in A \text{ such that } \beta(a) = s
]
\end{definition}

\section{Error Taxonomy}

\subsection{Contradiction}

[
T \models \bot
]

\subsection{Hallucination}

[
C_l({s}) = 1 \wedge \neg \exists a \in A : \beta(a) = s
]

\subsection{Anomaly}

[
P(x) < \epsilon
]

\section{Retention}

\begin{definition}
Retention is a mapping:
[
\rho(d) = \langle \rho_L, \rho_S, \rho_\Sigma, \rho_\Phi \rangle
]
\end{definition}

\begin{proposition}
[
\rho(d) \geq \theta \Rightarrow \mathrm{Coh}(d) \text{ is preserved}
]
\end{proposition}

\section{Computational Interpretation}

Computational coherence corresponds to tractable verification.

\begin{proposition}
[
C_c(T) \approx \text{verification in } \mathbf{P}
]
\end{proposition}

\section{Conclusion}

We presented a framework where coherence is treated as a structural invariant of semantic systems. This enables the construction of systems in which meaning is compiled, preserved, and audited.

\end{document}
