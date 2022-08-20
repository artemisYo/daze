import scala.collection.mutable.ArrayBuffer
import scala.collection.immutable.Vector
import util.control.Breaks._
import scala.io.StdIn

// TODO:
//   1. Decimal number parser, aka. floating point support
//   2. Think about punctuation parser
//   3. Combinators having some fancy syntax like "parser1 + parser2" being equivalent to "Choice(Vector(parser1, parser2))"

trait Tag {
  val isErr: Boolean = false
}
case class ErrorTag(error: String) extends Tag {override val isErr: Boolean = true}
case class StrTag(inner: String) extends Tag
case class NumTag(inner: Int) extends Tag
case class DigTag(inner: Byte) extends Tag

trait Parser {
  def run(inputState: ParsingState): ParsingState
}

class ParsingState(val input: String, val index: Int = 0, val results: ArrayBuffer[Tag] = ArrayBuffer[Tag]()) {
  override def toString(): String = {
    s"input:   $input,\nindex:   $index,\nresults: $results"
  }
}

class Str(token: String)(tag: Tag = StrTag(token), delimiter: Char = ' ', doDelimSkip: Boolean = true) extends Parser {
  def run(input: String, index: Int = 0): (String, Int, Tag) = {
    if input.length() <= index then
      (input, index, ErrorTag("End of string reached"))
    else if input.startsWith(token, index) then
      var newIndex = index + token.length()
      while input.length() > newIndex && input.charAt(newIndex) == delimiter && doDelimSkip do newIndex += 1
      (input, newIndex, tag)
    else
      (input, index, ErrorTag(s"Failed to match $token"))
  }
  def run(inputState: ParsingState): ParsingState = {
    val out = run(inputState.input, inputState.index)
    ParsingState(input = out(0), index = out(1), results = inputState.results += out(2))
  }
}

class Num(delimiter: Char = ' ', doDelimSkip: Boolean = true)(groupStop: Char = '.', doGroupStop: Boolean = true) extends Parser {
  def run(input: String, index: Int = 0): (String, Int, Tag) = {
    if input.length() <= index then
      (input, index, ErrorTag("End of string reached"))
    else
      var newIndex = index
      while (input.length() > newIndex && (input.charAt(newIndex).isDigit || (input.charAt(newIndex) == groupStop && doGroupStop))) {
        newIndex += 1
      }
      val result = input.substring(index, newIndex).filterNot(_ == groupStop).toInt
      while input.length() > newIndex && input.charAt(newIndex) == delimiter && doDelimSkip do newIndex += 1
      (input, newIndex, NumTag(result))
  }
  def run(inputState: ParsingState): ParsingState = {
    val out = run(inputState.input, inputState.index)
    ParsingState(input = out(0), index = out(1), results = inputState.results += out(2))
  }
}

class Dig(delimiter: Char = ' ', doDelimSkip: Boolean = true)() extends Parser {
  def run(input: String, index: Int = 0): (String, Int, Tag) = {
    if input.length() <= index then
      (input, index, ErrorTag("End of string reached"))
    else if input.charAt(index).isDigit then
      var newIndex = index+1
      while input.length() > newIndex && input.charAt(newIndex) == delimiter && doDelimSkip do newIndex += 1
      (input, newIndex, DigTag((input.charAt(index).toInt-48).toByte)) //?? For some reason this, even with .toInt.toByte, converts to 48+digit?
    else
      (input, index, ErrorTag("Failed to match digit"))
  }
  def run(inputState: ParsingState): ParsingState = {
    val out = run(inputState.input, inputState.index)
    ParsingState(input = out(0), index = out(1), results = inputState.results += out(2))
  }
}

class Choice(val cases: Vector[Parser]) extends Parser {
  def run(inputState: ParsingState): ParsingState = {
    var temp = inputState
    breakable {
      for (parser <- cases) {
        temp = parser.run(inputState)
        if !temp.results.last.isErr then
          break
        temp.results.remove(temp.results.length-1)
      }
    }
    temp
  }
}

class Constraint(val constraints: Vector[Parser]) extends Parser {
  def run(inputState: ParsingState): ParsingState = {
    var temp = inputState
    var counter = 0
    breakable {
      for (parser <- constraints) {
        counter += 1
        temp = parser.run(temp)
        if temp.results.last.isErr then
          temp.results.remove(temp.results.length-counter, counter)
          break
      }
    }
    temp
  }
}

@main def hello: Unit = {
  var state = ParsingState("Hello there!")
  val par1 = Str("Hello")()
  val par2 = Str("there")()
  val par  = Choice(Vector(par1, par2))
  println(par.run(par.run(state)))
}
