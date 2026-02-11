<!-- POWERBI TEMPLATE -->
Analyseer deze PowerBI (.py) Notebook en schrijf in Markdown documentatie voor onze Azure DevOps Wiki omgeving:

1. De opgeleverde dim of fact in twee tabellen;
    - Functioneel (Voor de BI experts)
    - Technisch (Voor de Data Engineers)
    - Mermaid UML (Functionele data flow met PowerBI-specifieke lagen)
2. Zorg er voor dat de tabellen als markdown tabellen worden gegenereerd.
3. Houd deze Markdown template aan:

| ʕ•ᴥ•ʔ                    | **Details**             |
| -------------------------- | ----------------------- |
| **Auteur**                 | Stefan-GPT              |
| **Notebook naam**          | {Notebook bestandsnaam} |
| **Platform**               | Microsoft PowerBI        |
| **Datum aanmaak document** | {Huidige datum en tijd} |

---

# 📚 Functionele Notebook omschrijving
{beschrijf hier duidelijk op een functionele manier wat deze code doet, met specifieke aandacht voor PowerBI Lakehouse architectuur en medallion layers}

---

## 🏗️ PowerBI Architectuur Context
{Beschrijf de positie in de medallion architectuur: Bronze/Silver/Gold layer}
{Beschrijf de relatie met OneLake en PowerBI workspace}

---

## 🧙‍♀️ UML Flow-chart
{Voeg een UML Mermaid `flowchart LR` hieronder toe op basis van dit template voorbeeld.}
{VOEG GEEN WHITESPACES EN SPATIES TOE AAN JOUW REACTIE!}

```Mermaid
flowchart LR

  %% ===== Styles =====
  classDef src fill:#EEF2FF,stroke:#4F46E5,color:#1E1B4B;
  classDef tf fill:#FFF7ED,stroke:#F59E0B,color:#78350F;
  classDef dq fill:#F0F9FF,stroke:#0EA5E9,color:#0C4A6E;
  classDef sink fill:#ECFDF5,stroke:#10B981,color:#064E3B;

  %% ===== Bron =====
  subgraph S[Bron]

    S1["<bronlaag>.<brontabel>"]:::src

  end

  %% ===== Transformaties =====
  subgraph T[Transformaties]

    T1["<Stapnaam> - <kerntransformatie>"]:::tf
    T2["<Stapnaam> - <join / filter / SCD / aggregatie>"]:::tf

  end

  %% ===== Checks (optioneel) =====
  subgraph Q[Checks]

    Q1["<Check> - <regel>"]:::dq

  end

  %% ===== Output =====
  subgraph O[Output]

    O1["<doellaag>.<doeltabel>"]:::sink

  end

  %% ===== Flow =====
  S1 --> T1 --> T2 --> Q1 --> O1
```

---

## 🧠 Functioneel ontwerp

| **Attribuutnaam** | **Definitie**                                                                                                     | **Omschrijving transformatie**                                                                                 |
|-------------------|-------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------|
| `dim_project_fk`  | De foreign key naar **dim_project**. Gebruik de definitie uit de prompt indien beschikbaar; anders logisch afgeleid uit de notebook. | De surrogate key (SK) van `dim_project_t` wordt geselecteerd en ge-aliast naar `dim_project_fk`. De data blijft verder ongewijzigd. |
|                   |                                                                                                                   |                                                                                                                |

---

## 🛠️ Technisch ontwerp

| **Attribuut naam**         | **Data Type**     | **Key**       | **Bron**                   | **Brontabel(en)**             | **Bronattribuut(en)**     | **Voorwaarde**                                                                     |
| ------------------------- | ----------------- | ------------- | -------------------------- | ----------------------------- | ------------------------- | ---------------------------------------------------------------------------------- |
| voorbeeld: `dim_project_fk` | voorbeeld: string | voorbeeld: Ja | voorbeeld: Staff-Lakehouse | voorbeeld: gold.dim_project_t | voorbeeld: dim_project_sk | voorbeeld: F.col("dim_project_sk").cast("string").alias("dim_project_fk"),         |
|                           |                   |               |                            |                               |                           |                                                                                    |

---

## ✅ Afsluiting

Deze documentatie is automatisch gegenereerd op basis van de PowerBI notebooklogica en dient als startpunt voor review door Data Engineering en BI. Eventuele aanvullingen, correcties of optimalisaties kunnen direct in deze Wiki worden doorgevoerd.

<p align="center">
🚀 <em>Samen zorgen we voor consistente, uitlegbare en onderhoudbare data-producten in Microsoft PowerBI.</em>
</p>