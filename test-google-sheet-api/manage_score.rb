require 'google/apis/sheets_v4'
require 'googleauth'
require 'googleauth/stores/file_token_store'
require 'fileutils'

OOB_URI = 'urn:ietf:wg:oauth:2.0:oob'.freeze
APPLICATION_NAME = 'Google Sheets API Ruby Quickstart'.freeze
CREDENTIALS_PATH = 'credentials.json'.freeze
# The file token.yaml stores the user's access and refresh tokens, and is
# created automatically when the authorization flow completes for the first
# time.
TOKEN_PATH = 'token.yaml'.freeze
# SCOPE = Google::Apis::SheetsV4::AUTH_SPREADSHEETS_READONLY
SCOPE = Google::Apis::SheetsV4::AUTH_SPREADSHEETS

##
# Ensure valid credentials, either by restoring from the saved credentials
# files or intitiating an OAuth2 authorization. If authorization is required,
# the user's default browser will be launched to approve the request.
#
# @return [Google::Auth::UserRefreshCredentials] OAuth2 credentials
def authorize
  client_id = Google::Auth::ClientId.from_file(CREDENTIALS_PATH)
  token_store = Google::Auth::Stores::FileTokenStore.new(file: TOKEN_PATH)
  authorizer = Google::Auth::UserAuthorizer.new(client_id, SCOPE, token_store)
  user_id = 'default'
  credentials = authorizer.get_credentials(user_id)
  if credentials.nil?
    url = authorizer.get_authorization_url(base_url: OOB_URI)
    puts 'Open the following URL in the browser and enter the ' \
         "resulting code after authorization:\n" + url
    code = gets
    credentials = authorizer.get_and_store_credentials_from_code(
      user_id: user_id, code: code, base_url: OOB_URI
    )
  end
  credentials
end

# Prints all score tab
def read_score_table service, spreadsheet_id
    response = service.get_spreadsheet_values(spreadsheet_id, 'Score!A:B')
    puts 'No data found.' if response.values.empty?
    response.values.each do |row|
    puts "#{row[0]}, #{row[1]}"
    end
end

def previous_scores_nb service, spreadsheet_id
    response = service.get_spreadsheet_values(spreadsheet_id, 'Score!A:B')
    response.values.length - 1
end

def write_score service, spreadsheet_id, name, score
    range = 'Score!A3:B4'
    values = [ [name, score] ]
    # data = [
    #     {
    #     range: range,
    #     values: values
    #     },
    # ]

    cell_num = previous_scores_nb(service, spreadsheet_id) + 2
    range = "Score!A#{cell_num}:B#{cell_num}"
    value_range_object = Google::Apis::SheetsV4::ValueRange.new(range: range, values: values)
    result = service.update_spreadsheet_value(
        spreadsheet_id,
        range,
        value_range_object,
        value_input_option: "USER_ENTERED"
    )
    # puts "#{result.updated_cells} cells updated."
    puts "Score added => (#{name}, #{score})"
end

# Initialize the API
service = Google::Apis::SheetsV4::SheetsService.new
service.client_options.application_name = APPLICATION_NAME
service.authorization = authorize

# https://docs.google.com/spreadsheets/d/14QCM9etTdXXc1U2zg5EaX-VcLW04Tzo5lOqWaPnzrQE/edit
spreadsheet_id = '14QCM9etTdXXc1U2zg5EaX-VcLW04Tzo5lOqWaPnzrQE'

puts "Read all score table:"
read_score_table service, spreadsheet_id
write_score service, spreadsheet_id, "Clem", 21


