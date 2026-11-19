/** Editorial crosswalk, not a statement that entire PSC units are covered by NEC.
 * Labels are shortened from the existing chapter maps. Official scope is linked on the dashboard. */
export interface StudyTopic { id: string; title: string; lesson: string; ce: number | null; co: number | null }
export interface StudyChapter { id: number; title: string; path: string; topics: StudyTopic[] }
export const studyChapters: StudyChapter[] = [
  {
    "id": 1,
    "title": "Basic electrical & electronics",
    "path": "nec/chapters/01-electrical-electronics/",
    "topics": [
      {
        "id": "1.1",
        "title": "Basic concepts",
        "lesson": "nec/electronics/00-start-here/",
        "ce": 4,
        "co": null
      },
      {
        "id": "1.2",
        "title": "Network theorems",
        "lesson": "nec/electronics/00-start-here/",
        "ce": 4,
        "co": null
      },
      {
        "id": "1.3",
        "title": "AC fundamentals",
        "lesson": "nec/electronics/00-start-here/",
        "ce": 4,
        "co": null
      },
      {
        "id": "1.4",
        "title": "Semiconductor devices",
        "lesson": "nec/electronics/00-start-here/",
        "ce": 4,
        "co": null
      },
      {
        "id": "1.5",
        "title": "Signal generators",
        "lesson": "nec/electronics/00-start-here/",
        "ce": 4,
        "co": null
      },
      {
        "id": "1.6",
        "title": "Amplifiers",
        "lesson": "nec/electronics/00-start-here/",
        "ce": 4,
        "co": null
      }
    ]
  },
  {
    "id": 2,
    "title": "Digital logic & microprocessor",
    "path": "nec/chapters/02-digital-logic-microprocessor/",
    "topics": [
      {
        "id": "2.1",
        "title": "Digital logic",
        "lesson": "nec/digital/00-start-here/",
        "ce": 3,
        "co": 8
      },
      {
        "id": "2.2",
        "title": "Combinational and arithmetic circuits",
        "lesson": "nec/digital/00-start-here/",
        "ce": 3,
        "co": 8
      },
      {
        "id": "2.3",
        "title": "Sequential logic circuits",
        "lesson": "nec/digital/00-start-here/",
        "ce": 3,
        "co": 8
      },
      {
        "id": "2.4",
        "title": "Microprocessor",
        "lesson": "nec/digital/00-start-here/",
        "ce": 2,
        "co": 8
      },
      {
        "id": "2.5",
        "title": "Microprocessor system",
        "lesson": "nec/digital/00-start-here/",
        "ce": 2,
        "co": 8
      },
      {
        "id": "2.6",
        "title": "Interrupt operations",
        "lesson": "nec/digital/00-start-here/",
        "ce": 2,
        "co": 8
      }
    ]
  },
  {
    "id": 3,
    "title": "Programming in C & C++",
    "path": "nec/chapters/03-programming-c-cpp/",
    "topics": [
      {
        "id": "3.1",
        "title": "C fundamentals",
        "lesson": "nec/cpp/00-start-here/",
        "ce": 6,
        "co": 6
      },
      {
        "id": "3.2",
        "title": "Pointers, structures and files",
        "lesson": "nec/cpp/00-start-here/",
        "ce": 6,
        "co": 6
      },
      {
        "id": "3.3",
        "title": "Classes and objects",
        "lesson": "nec/cpp/00-start-here/",
        "ce": 6,
        "co": 6
      },
      {
        "id": "3.4",
        "title": "Overloading and inheritance",
        "lesson": "nec/cpp/00-start-here/",
        "ce": 6,
        "co": 6
      },
      {
        "id": "3.5",
        "title": "Virtual functions and streams",
        "lesson": "nec/cpp/00-start-here/",
        "ce": 6,
        "co": 6
      },
      {
        "id": "3.6",
        "title": "Templates and exceptions",
        "lesson": "nec/cpp/00-start-here/",
        "ce": 6,
        "co": 6
      }
    ]
  },
  {
    "id": 4,
    "title": "Computer organization & embedded systems",
    "path": "nec/chapters/04-computer-organization-embedded/",
    "topics": [
      {
        "id": "4.1",
        "title": "Control and central processing units",
        "lesson": "nec/organization/00-start-here/",
        "ce": 2,
        "co": 8
      },
      {
        "id": "4.2",
        "title": "Computer arithmetic and memory system",
        "lesson": "nec/organization/00-start-here/",
        "ce": 2,
        "co": 8
      },
      {
        "id": "4.3",
        "title": "Input–output organization and multiprocessors",
        "lesson": "nec/organization/00-start-here/",
        "ce": 2,
        "co": 8
      },
      {
        "id": "4.4",
        "title": "Embedded systems: hardware–software design",
        "lesson": "nec/preparation/03-embedded-systems-rtos-vhdl/",
        "ce": 15,
        "co": null
      },
      {
        "id": "4.5",
        "title": "Real-time operating and control systems",
        "lesson": "nec/preparation/03-embedded-systems-rtos-vhdl/",
        "ce": 15,
        "co": null
      },
      {
        "id": "4.6",
        "title": "Hardware description language and IC technology",
        "lesson": "nec/preparation/03-embedded-systems-rtos-vhdl/",
        "ce": 15,
        "co": null
      }
    ]
  },
  {
    "id": 5,
    "title": "Networks & security",
    "path": "nec/chapters/05-networks-security/",
    "topics": [
      {
        "id": "5.1",
        "title": "Network models and media",
        "lesson": "nec/networks/00-start-here/",
        "ce": 1,
        "co": 7
      },
      {
        "id": "5.2",
        "title": "Link layer and LANs",
        "lesson": "nec/networks/00-start-here/",
        "ce": 1,
        "co": 7
      },
      {
        "id": "5.3",
        "title": "IP, subnetting and routing",
        "lesson": "nec/networks/00-start-here/",
        "ce": 1,
        "co": 7
      },
      {
        "id": "5.4",
        "title": "Transport and congestion",
        "lesson": "nec/networks/00-start-here/",
        "ce": 1,
        "co": 7
      },
      {
        "id": "5.5",
        "title": "Application protocols",
        "lesson": "nec/networks/00-start-here/",
        "ce": 1,
        "co": 7
      },
      {
        "id": "5.6",
        "title": "Network security",
        "lesson": "nec/networks/00-start-here/",
        "ce": 1,
        "co": 7
      }
    ]
  },
  {
    "id": 6,
    "title": "Theory of computation & graphics",
    "path": "nec/chapters/06-toc-graphics/",
    "topics": [
      {
        "id": "6.1",
        "title": "Finite automata",
        "lesson": "nec/preparation/01-automata-and-graphics-traces/",
        "ce": 12,
        "co": null
      },
      {
        "id": "6.2",
        "title": "Context-free languages",
        "lesson": "nec/preparation/06-cfl-and-parametric-clipping/",
        "ce": 12,
        "co": null
      },
      {
        "id": "6.3",
        "title": "Turing machines",
        "lesson": "nec/preparation/01-automata-and-graphics-traces/",
        "ce": 12,
        "co": null
      },
      {
        "id": "6.4",
        "title": "Graphics hardware and software",
        "lesson": "nec/preparation/01-automata-and-graphics-traces/",
        "ce": 14,
        "co": null
      },
      {
        "id": "6.5",
        "title": "2D transformations and clipping",
        "lesson": "nec/preparation/06-cfl-and-parametric-clipping/",
        "ce": 14,
        "co": null
      },
      {
        "id": "6.6",
        "title": "3D transformations and projections",
        "lesson": "nec/preparation/01-automata-and-graphics-traces/",
        "ce": 14,
        "co": null
      }
    ]
  },
  {
    "id": 7,
    "title": "DSA, DBMS & OS",
    "path": "nec/chapters/07-dsa-dbms-os/",
    "topics": [
      {
        "id": "7.1",
        "title": "Lists, stacks, queues and trees",
        "lesson": "nec/dsa/00-start-here/",
        "ce": 7,
        "co": 2
      },
      {
        "id": "7.2",
        "title": "Sorting, hashing and graphs",
        "lesson": "nec/dsa/00-start-here/",
        "ce": 7,
        "co": 2
      },
      {
        "id": "7.3",
        "title": "Models, normalisation and SQL",
        "lesson": "nec/dbms/00-start-here/",
        "ce": 9,
        "co": 5
      },
      {
        "id": "7.4",
        "title": "Transactions and recovery",
        "lesson": "nec/dbms/00-start-here/",
        "ce": 9,
        "co": 5
      },
      {
        "id": "7.5",
        "title": "Processes and synchronisation",
        "lesson": "nec/os/00-start-here/",
        "ce": 10,
        "co": 4
      },
      {
        "id": "7.6",
        "title": "Memory, files and administration",
        "lesson": "nec/os/00-start-here/",
        "ce": 10,
        "co": 4
      }
    ]
  },
  {
    "id": 8,
    "title": "Software engineering & OOAD",
    "path": "nec/chapters/08-software-engineering-ooad/",
    "topics": [
      {
        "id": "8.1",
        "title": "Software process and requirements",
        "lesson": "nec/se/00-start-here/",
        "ce": 8,
        "co": 3
      },
      {
        "id": "8.2",
        "title": "Software design",
        "lesson": "nec/se/00-start-here/",
        "ce": 8,
        "co": 3
      },
      {
        "id": "8.3",
        "title": "Testing, estimation, quality and configuration management",
        "lesson": "nec/se/00-start-here/",
        "ce": 8,
        "co": 3
      },
      {
        "id": "8.4",
        "title": "OO fundamentals and analysis",
        "lesson": "nec/se/00-start-here/",
        "ce": 8,
        "co": 3
      },
      {
        "id": "8.5",
        "title": "OO design",
        "lesson": "nec/se/00-start-here/",
        "ce": 8,
        "co": 3
      },
      {
        "id": "8.6",
        "title": "OO design implementation",
        "lesson": "nec/se/00-start-here/",
        "ce": 8,
        "co": 3
      }
    ]
  },
  {
    "id": 9,
    "title": "AI & neural networks",
    "path": "nec/chapters/09-ai-neural-networks/",
    "topics": [
      {
        "id": "9.1",
        "title": "AI and intelligent agents",
        "lesson": "nec/preparation/02-ai-search-and-neural-networks/",
        "ce": 11,
        "co": null
      },
      {
        "id": "9.2",
        "title": "Search",
        "lesson": "nec/preparation/02-ai-search-and-neural-networks/",
        "ce": 11,
        "co": null
      },
      {
        "id": "9.3",
        "title": "Knowledge representation",
        "lesson": "nec/preparation/07-probability-and-learning-traces/",
        "ce": 11,
        "co": null
      },
      {
        "id": "9.4",
        "title": "Expert systems and NLP",
        "lesson": "nec/preparation/02-ai-search-and-neural-networks/",
        "ce": 11,
        "co": null
      },
      {
        "id": "9.5",
        "title": "Machine learning",
        "lesson": "nec/preparation/07-probability-and-learning-traces/",
        "ce": 11,
        "co": null
      },
      {
        "id": "9.6",
        "title": "Neural networks",
        "lesson": "nec/preparation/07-probability-and-learning-traces/",
        "ce": 15,
        "co": null
      }
    ]
  },
  {
    "id": 10,
    "title": "Project planning, design & implementation",
    "path": "nec/chapters/10-project-planning/",
    "topics": [
      {
        "id": "10.1",
        "title": "Engineering drawing",
        "lesson": "nec/preparation/05-engineering-drawing-and-professional-practice/",
        "ce": null,
        "co": null
      },
      {
        "id": "10.2",
        "title": "Engineering economics",
        "lesson": "nec/preparation/04-project-economics-and-cpm/",
        "ce": null,
        "co": null
      },
      {
        "id": "10.3",
        "title": "Project planning and scheduling",
        "lesson": "nec/preparation/04-project-economics-and-cpm/",
        "ce": 15,
        "co": null
      },
      {
        "id": "10.4",
        "title": "Project management",
        "lesson": "nec/preparation/05-engineering-drawing-and-professional-practice/",
        "ce": 15,
        "co": null
      },
      {
        "id": "10.5",
        "title": "Engineering professional practice",
        "lesson": "nec/preparation/05-engineering-drawing-and-professional-practice/",
        "ce": null,
        "co": null
      },
      {
        "id": "10.6",
        "title": "Engineering regulatory body — NEC Act and Regulations",
        "lesson": "nec/preparation/05-engineering-drawing-and-professional-practice/",
        "ce": null,
        "co": null
      }
    ]
  }
];
export const studyTopics = studyChapters.flatMap(chapter => chapter.topics.map(topic => ({...topic, chapter: chapter.id})));
