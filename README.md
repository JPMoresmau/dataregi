# DataRegi

A little web site to exercise Rust as a server side language. 


The business goal is to help with the proliferation of spreadsheets in the enterprise, by allowing people to find multiple versions of the same spreadsheet.
The main feature is to extract documents from emails that are sent to a email address the web site listens to.


The server side is built on Rust with Rocket and Diesel, and relies on AWS APIs to send mail and receive queued messages.


The client side is HTML/CSS with plain Javascript.