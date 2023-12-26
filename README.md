# 🏔️ Tundra

Tundra provides a framework and utilities for creating interactive terminal applications with Ratatui. 

Ratatui is a comprehensive library for creating singular user interfaces in the terminal, but lacks features
for organizing larger applications — composed of several interfaces — and for receiving user data input. 

Tundra aims to extend the functionality of Ratatui with utilities for: 

- Defining application states. 
- Managing the terminal environment and context. 
- Displaying messages through modal dialogs. 
- Receiving user input through input forms and fields. 

It is **not** intended to be a replacement for or wrapper over Ratatui, nor the backend. Ratatui is still
required to draw the user interface of each application state, and the backend is still required for
low-level terminal operations. 


## 📚 Documentation

Read the documentation at TODO. 


## 🪧 A Note on the Backend
 
Ratatui has support for several terminal backends. If you don't know what that means, this holds no
significance to you. 
 
Tundra currently only supports the crossterm backend. This is due to a lack of abstraction over the different
backends. Code — particularly pertaining to context and event handling — would have to be written and
repeated for each backend. 
 
If you need another backend for your project, Tundra is not for you — at least for the moment. 
