import scala.collection.mutable.ArrayBuffer
import scala.collection.immutable.Vector
import util.control.Breaks._
import scala.io.StdIn

// TODO:
//   1. Decimal number parser, aka. floating point support
//   2. Think about punctuation parser

trait Tag {
  val isErr: Boolean = false
}
case class ErrorTag(error: String) extends Tag {override val isErr: Boolean = true}
case class StrTag(inner: String) extends Tag
case class Any(inner: String) extends Tag
case class NumTag(inner: Int) extends Tag
case class DigTag(inner: Byte) extends Tag

// this is a hack to make Constraint be able to return Tag eventhough it has multiple results
// NOTE: functions which return this are unreliable cuz yeah
case class ManyTag(inner: ArrayBuffer[Tag]) extends Tag

case class Exit() extends Tag
case class Print() extends Tag
case class Save() extends Tag
case class Append() extends Tag
case class Insert() extends Tag
case class Delete() extends Tag
case class Set() extends Tag
case class Relations() extends Tag
case class Name() extends Tag
case class Value() extends Tag
case class PrintGraph() extends Tag
case class BoolLit(inner: Boolean) extends Tag

trait Parser {
  def run(input: String, index: Int): (String, Int, Tag)
  def run(inputState: ParsingState): ParsingState = {
    val out = run(inputState.input, inputState.index)
    ParsingState(input = out(0), index = out(1), results = inputState.results += out(2))
  }
  def &(that: Parser): Constraint = this match
  case x: Constraint => Constraint(x.constraints :+ that)
  case other => Constraint(Vector(this, that))

  def |(that: Parser): Choice = this match
  case x: Choice => Choice(x.cases :+ that)
  case other => Choice(Vector(this, that))

  def count: Count = Count(this)
}

def *:(parser: Parser): Count = parser.count

class ParsingState(val input: String, var index: Int = 0, val results: ArrayBuffer[Tag] = ArrayBuffer[Tag]()) {
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
}

class AnyStr(delimiter: Char = ' ', doDelimSkip: Boolean = true) extends Parser {
  def run(input: String, index: Int = 0): (String, Int, Tag) = {
    if index == input.length() then
      (input, index, ErrorTag("Reached end of string"))
    else
      var newIndex = input.indexOf(delimiter, index)
      if newIndex == -1 then
        newIndex = input.length()
        (input, newIndex, Any(input.substring(index, newIndex)))
      else
        val out = input.substring(index, newIndex)
        while newIndex < input.length() && input.charAt(newIndex) == delimiter do newIndex += 1
        (input, newIndex, Any(out))
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
      val temp = input.substring(index, newIndex).filterNot(_ == groupStop)
      if temp.isEmpty then
        (input, index, ErrorTag("Could not Match"))
      else
        val result = input.substring(index, newIndex).filterNot(_ == groupStop).toInt
        while input.length() > newIndex && input.charAt(newIndex) == delimiter && doDelimSkip do newIndex += 1
        (input, newIndex, NumTag(result))
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
}
class Choice(var cases: Vector[Parser]) extends Parser {
  def run(input: String, index: Int = 0): (String, Int, Tag) = {
    var temp: (String, Int, Tag) = (input, index, ErrorTag("Could not Match"))
    breakable {
      for (parser <- cases) {
        temp = parser.run(temp(0), temp(1))
        if !temp(2).isErr then
          break
      }
    }
    temp
  }
  override def run(inputState: ParsingState): ParsingState = {
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

class Constraint(var constraints: Vector[Parser]) extends Parser {
  def run(input: String, index: Int = 0): (String, Int, Tag) = {
    var temp = run(ParsingState(input, index))
    (temp.input, temp.index, ManyTag(temp.results))
  }
  override def run(inputState: ParsingState): ParsingState = {
    var temp = inputState
    val oldIndex = inputState.index
    var counter = 0
    breakable {
      for (parser <- constraints) {
        counter += 1
        temp = parser.run(temp)
        if temp.results.last.isErr then
          temp.index = oldIndex
          temp.results.remove(temp.results.length-counter, counter)
          temp.results += ErrorTag("Could not Match")
          break
      }
    }
    temp
  }
}

class Count(val parser: Parser) extends Parser {
  def run(input: String, index: Int = 0): (String, Int, Tag) = {
    var temp = run(ParsingState(input, index))
    (temp.input, temp.index, ManyTag(temp.results))
  }
  override def run(inputState: ParsingState): ParsingState = {
    var temp = inputState
    breakable {
      while (true) {
        temp = parser.run(temp)
        if temp.results.last.isErr then
          temp.results.remove(temp.results.length-1)
          break
      }
    }
    temp
  }
}

@main def hello: Unit = {
  val value = Str("true")(BoolLit(true)) | Str("false")(BoolLit(false)) | Num()() | AnyStr()
  val par = Str("exit")(Exit())
            | Str("print")(Print())
            | Str("graphviz")(PrintGraph())
            | Str("save")(Save()) & AnyStr()
            | Str("save")(Save())
            | Str("append")(Append()) & AnyStr() & value
            | Str("insert")(Insert()) & AnyStr() & value & Num()()
            | Str("delete")(Delete()) & Num()()
            | Str("set")(Set()) & (Str("relations")(Relations()) & *:(Num()()) | Str("name")(Name()) & AnyStr() | Str("value")(Value()) & value)
  breakable {
    while (true) {
      val input = StdIn.readLine("    $> ")
      println(input)
      // TODO: this match thingy cuz it's the logic of the program
      val state = par.run(ParsingState(input))
      for (tag <- state.results) {
        tag match
        case Exit() => break
        case ErrorTag(e) => println(s"[Error]: Error occured: $e")
        case other => println(s"[Error]: Could not recognise $other")
      }
    }
  }
}
