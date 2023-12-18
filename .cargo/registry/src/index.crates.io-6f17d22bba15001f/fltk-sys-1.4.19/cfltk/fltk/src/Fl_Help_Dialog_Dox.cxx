//
// Fl_Help_Dialog dialog for the Fast Light Tool Kit (FLTK).
//
// Copyright 1998-2022 by Bill Spitzak and others.
//
// This library is free software. Distribution and use rights are outlined in
// the file "COPYING" which should have been included with this file.  If this
// file is missing or damaged, see the license at:
//
//     https://www.fltk.org/COPYING.php
//
// Please see the following page on how to report bugs and issues:
//
//     https://www.fltk.org/bugs.php
//

// Fl_Help_Dialog (autogenerated class) doxygen documentation placeholder

/** \class Fl_Help_Dialog
  The Fl_Help_Dialog widget displays a standard help dialog window
  using the Fl_Help_View widget.

  The Fl_Help_Dialog class is not part of the FLTK core library, but instead
  of \em fltk_images. Use \c --use-images when compiling with \c fltk-config .

  \image html  Fl_Help_Dialog.png </P>
  \image latex Fl_Help_Dialog.png "Fl_Help_Dialog" width=8cm
*/

/** \fn Fl_Help_Dialog::Fl_Help_Dialog()
  The constructor creates the dialog pictured above.*/

/** \fn Fl_Help_Dialog::~Fl_Help_Dialog()
  The destructor destroys the widget and frees all memory that has been
  allocated for the current file.
*/

/** \fn void Fl_Help_Dialog::hide()
  Hides the Fl_Help_Dialog window.*/

/** \fn int Fl_Help_Dialog::load(const char *f)
 \brief Loads the specified HTML file into the Fl_Help_View widget.
 The filename can also contain a target name ("filename.html#target"). Always
 use forward slashes as path deliminators, MSWindows-style backslashes are not
 supported here
 \param[in] f the name and path of an HTML file
 \return 0 on success, -1 on error
 \see Fl_Help_View::load(), fl_load_uri()
*/

/** \fn void Fl_Help_Dialog::position(int x, int y)
  Set the screen position of the dialog.*/

/** \fn void Fl_Help_Dialog::resize(int xx, int yy, int ww, int hh)
  Change the position and size of the dialog.*/

/** \fn void Fl_Help_Dialog::show()
  Shows the Fl_Help_Dialog window.*/

/** \fn void Fl_Help_Dialog::textsize(Fl_Fontsize s)
  Sets or gets the default text size for the help view.*/

/** \fn uchar Fl_Help_Dialog::textsize()
  Sets or gets the default text size for the help view.*/

/** \fn void Fl_Help_Dialog::topline(const char *n)
  Sets the top line in the Fl_Help_View widget to the named or
  numbered line.
*/

/** \fn void Fl_Help_Dialog::topline(int n)
  Sets the top line in the Fl_Help_View widget to the named or
  numbered line.
*/

/** \fn void Fl_Help_Dialog::value(const char *v)
  The first form sets the current buffer to the string provided and
  reformats the text. It also clears the history of the "back" and
  "forward" buttons. The second form returns the current buffer contents.
*/

/** \fn const char *Fl_Help_Dialog::value() const
  The first form sets the current buffer to the string provided and
  reformats the text. It also clears the history of the "back" and
  "forward" buttons. The second form returns the current buffer contents.
*/

/** \fn int Fl_Help_Dialog::visible()
  Returns 1 if the Fl_Help_Dialog window is visible.*/

/** \fn int Fl_Help_Dialog::x()
  Returns the position and size of the help dialog.*/

/** \fn int Fl_Help_Dialog::y()
  Returns the position and size of the help dialog.*/

/** \fn int Fl_Help_Dialog::w()
  Returns the position and size of the help dialog.*/

/** \fn int Fl_Help_Dialog::h()
  Returns the position and size of the help dialog.*/

/** \fn void Fl_Help_Dialog::show()
  Shows the main Help Dialog Window
  Delegates call to encapsulated window_ void Fl_Window::show() method */

/** \fn void Fl_Help_Dialog::show(int argc, char **argv)
  Shows the main Help Dialog Window
  Delegates call to encapsulated window_ void Fl_Window::show(int argc, char **argv) instance method */

/** \fn void Fl_Help_Dialog::textsize(Fl_Fontsize s)
  Sets the internal Fl_Help_View instance text size.
  Delegates call to encapsulated view_ void Fl_Help_View::textsize(Fl_Fontsize s) instance method */
